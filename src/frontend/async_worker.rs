use std::path::Path;

use futures::StreamExt;
use tokio::sync::RwLock;

use crate::{
    frontend::{utils::maybe_plural, work_item::WorkProgress},
    nodes::Block,
    rules::{bundle::Bundler, ContextBuilder, Rule, RuleConfiguration},
    utils::{normalize_path, Timer},
    Configuration, DarkluaError, Resources,
};

use super::{
    work_cache::WorkCache,
    work_item::{WorkData, WorkItem, WorkStatus},
    DarkluaResult, ProcessResult,
};

#[derive(Debug)]
pub(crate) struct AsyncWorker {
    resources: Resources,
    configuration: Configuration,
    cache: RwLock<WorkCache>,
    cached_bundler: RwLock<Option<Bundler>>,
}

impl AsyncWorker {
    pub fn new(resources: Resources, configuration: Configuration) -> Self {
        let cache = RwLock::new(WorkCache::new(&resources));
        Self {
            resources,
            configuration,
            cache,
            cached_bundler: RwLock::new(None),
        }
    }

    pub async fn process(
        self,
        mut work_items: Vec<WorkItem>,
        fail_fast: bool,
    ) -> Result<ProcessResult, ProcessResult> {
        let mut success_count = 0;
        let mut errors = Vec::new();

        'work_loop: while !work_items.is_empty() {
            let work_length = work_items.len();

            let mut work_futures = futures::stream::FuturesUnordered::new();
            log::trace!(
                "working on batch of {} task{}",
                work_length,
                maybe_plural(work_length)
            );

            for work in work_items {
                work_futures.push(async {
                    let source = work.source().display().to_string();

                    (source, self.do_work(work).await)
                })
            }

            let mut work_left = Vec::new();

            while let Some((source, work_result)) = work_futures.next().await {
                match work_result {
                    Ok(None) => {
                        success_count += 1;
                        log::info!("successfully processed `{}`", source);
                    }
                    Ok(Some(next_work)) => {
                        log::trace!("work on `{}` has not completed", source);
                        work_left.push(next_work);
                    }
                    Err(err) => {
                        errors.push(err);
                        if fail_fast {
                            log::debug!(
                                "dropping all work because the fail-fast option is enabled"
                            );
                            break 'work_loop;
                        }
                    }
                }
            }

            if work_left.len() >= work_length {
                errors.push(DarkluaError::cyclic_work(work_left));
                return ProcessResult::new(success_count, errors).into();
            }

            work_items = work_left;
        }

        ProcessResult::new(success_count, errors).into()
    }

    async fn do_work(&self, work: WorkItem) -> DarkluaResult<Option<WorkItem>> {
        let (status, data) = work.extract();
        match status {
            WorkStatus::NotStarted => {
                let source_display = data.source().display();

                let source = data.source();
                let content = self.resources.get(source).await?;

                let parser = self.configuration.build_parser();

                log::debug!("beginning work on `{}`", source_display);

                let parser_timer = Timer::now();

                let mut block = parser
                    .parse(&content)
                    .map_err(|parser_error| DarkluaError::parser_error(source, parser_error))?;

                let parser_time = parser_timer.duration_label();
                log::debug!("parsed `{}` in {}", source_display, parser_time);

                self.bundle(&mut block, source, &content).await?;

                self.apply_rules(data, WorkProgress::new(content, block))
                    .await
            }
            WorkStatus::InProgress(progress) => self.apply_rules(data, *progress).await,
        }
    }

    async fn apply_rules(
        &self,
        data: WorkData,
        progress: WorkProgress,
    ) -> DarkluaResult<Option<WorkItem>> {
        let (content, mut progress) = progress.extract();

        let source_display = data.source().display();
        let normalized_source = normalize_path(data.source());

        progress.duration().start();

        for (index, rule) in self
            .configuration
            .rules()
            .enumerate()
            .skip(progress.next_rule())
        {
            let mut context_builder = self.create_rule_context(data.source(), &content);
            log::trace!(
                "[{}] apply rule `{}`{}",
                source_display,
                rule.get_name(),
                if rule.has_properties() {
                    format!(" {:?}", rule.serialize_to_properties())
                } else {
                    "".to_owned()
                }
            );
            let mut required_content: Vec<_> = rule
                .require_content(&normalized_source, progress.block())
                .into_iter()
                .map(normalize_path)
                .filter(|path| {
                    if *path == normalized_source {
                        log::debug!("filtering out currently processing path");
                        false
                    } else {
                        true
                    }
                })
                .collect();
            required_content.sort();
            required_content.dedup();

            let cache: tokio::sync::RwLockReadGuard<'_, WorkCache> = self.cache.read().await;

            if !required_content.is_empty() {
                if required_content.iter().all(|path| cache.contains(path)) {
                    let parser = self.configuration.build_parser();
                    for path in required_content.iter() {
                        let block = cache.get_block(path, &parser).await?;
                        context_builder.insert_block(path, block);
                    }
                } else {
                    progress.duration().pause();
                    log::trace!(
                        "queue work for `{}` at rule `{}` (#{}) because it requires:{}",
                        source_display,
                        rule.get_name(),
                        index,
                        if required_content.len() == 1 {
                            format!(" {}", required_content.first().unwrap().display())
                        } else {
                            format!(
                                "\n- {}",
                                required_content
                                    .iter()
                                    .map(|path| format!("- {}", path.display()))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            )
                        }
                    );
                    return Ok(Some(
                        data.with_status(
                            progress
                                .at_rule(index)
                                .with_required_content(required_content)
                                .with_content(content),
                        ),
                    ));
                }
            }

            let context = context_builder.build();
            let block = progress.mutate_block();
            let rule_timer = Timer::now();
            rule.process(block, &context).map_err(|rule_error| {
                let error = DarkluaError::rule_error(data.source(), rule, index, rule_error);

                log::trace!(
                    "[{}] rule `{}` errored: {}",
                    source_display,
                    rule.get_name(),
                    error
                );

                error
            })?;
            let rule_duration = rule_timer.duration_label();
            log::trace!(
                "[{}] ⨽completed `{}` in {}",
                source_display,
                rule.get_name(),
                rule_duration
            );
        }

        let rule_time = progress.duration().duration_label();
        let total_rules = self.configuration.rules_len();
        log::debug!(
            "{} rule{} applied in {} for `{}`",
            total_rules,
            maybe_plural(total_rules),
            rule_time,
            source_display,
        );

        log::trace!("begin generating code for `{}`", source_display);

        if cfg!(test) || (cfg!(debug_assertions) && log::log_enabled!(log::Level::Trace)) {
            log::trace!(
                "generate AST debugging view at `{}`",
                data.output().display()
            );
            self.resources
                .write(data.output(), &format!("{:#?}", progress.block())).await?;
        }

        let generator_timer = Timer::now();

        let lua_code = self.configuration.generate_lua(progress.block(), &content);

        let generator_time = generator_timer.duration_label();
        log::debug!(
            "generated code for `{}` in {}",
            source_display,
            generator_time,
        );

        self.resources.write(data.output(), &lua_code).await?;

        {
            self.cache
                .write()
                .await
                .link_source_to_output(normalized_source, data.output());
        }

        Ok(None)
    }

    async fn bundle(
        &self,
        block: &mut Block,
        source: &Path,
        original_code: &str,
    ) -> DarkluaResult<()> {
        if self.cached_bundler.read().await.is_none() {
            if let Some(bundler) = self.configuration.bundle() {
                let mut cached = self.cached_bundler.write().await;
                *cached = Some(bundler);
            }
        }

        let bundler_guard = &self.cached_bundler.read().await;
        let bundler = match bundler_guard.as_ref() {
            Some(bundler) => bundler,
            None => return Ok(()),
        };

        log::debug!("beginning bundling from `{}`", source.display());

        let bundle_timer = Timer::now();

        let context = self.create_rule_context(source, original_code).build();

        bundler.process(block, &context).map_err(|rule_error| {
            let error = DarkluaError::orphan_rule_error(source, bundler, rule_error);

            log::trace!(
                "[{}] rule `{}` errored: {}",
                source.display(),
                bundler.get_name(),
                error
            );

            error
        })?;

        let bundle_time = bundle_timer.duration_label();
        log::debug!("bundled `{}` in {}", source.display(), bundle_time);

        Ok(())
    }

    fn create_rule_context<'block, 'src>(
        &self,
        source: &Path,
        original_code: &'src str,
    ) -> ContextBuilder<'block, 'src> {
        let builder = ContextBuilder::new(normalize_path(source), &self.resources, original_code);
        if let Some(project_location) = self.configuration.location() {
            builder.with_project_location(project_location)
        } else {
            builder
        }
    }
}
