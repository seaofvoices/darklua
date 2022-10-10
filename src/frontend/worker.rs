use std::path::Path;

use crate::rules::ContextBuilder;

use super::{
    configuration::Configuration,
    resources::Resources,
    utils::{self, Timer},
    work_item::{Progress, WorkCache, WorkData, WorkItem, WorkStatus},
    DarkluaError, Options,
};

const DEFAULT_CONFIG_PATHS: [&str; 2] = [".darklua.json", ".darklua.json5"];

#[derive(Debug)]
pub struct Worker<'a> {
    resources: &'a Resources,
    cache: WorkCache<'a>,
    configuration: Configuration,
}

impl<'a> Worker<'a> {
    pub fn new(resources: &'a Resources) -> Self {
        Self {
            resources,
            cache: WorkCache::new(resources),
            configuration: Configuration::default(),
        }
    }

    pub fn process(
        mut self,
        work_items: impl Iterator<Item = Result<WorkItem, DarkluaError>>,
        mut options: Options,
    ) -> Result<(), Vec<DarkluaError>> {
        let configuration_setup_timer = Timer::now();

        if let Some(config) = options.take_configuration() {
            self.configuration = config;
            if let Some(config_path) = options.configuration_path() {
                log::warn!(
                    concat!(
                        "the provided options contained both a configuration object and ",
                        "a path to a configuration file (`{}`). the provided configuration ",
                        "takes precedence, so it is best to avoid confusion by providing ",
                        "only the configuration itself or a path to a configuration"
                    ),
                    config_path.display()
                );
            }
        } else if let Some(config) = options.configuration_path() {
            if self.resources.exists(config).map_err(element_to_vec)? {
                self.configuration = self.read_configuration(config).map_err(element_to_vec)?;
            } else {
                return Err(vec![DarkluaError::resource_not_found(config).context(
                    "expected to find configuration file as provided by the options",
                )]);
            }
        } else {
            let mut configuration_files = Vec::new();
            for path in DEFAULT_CONFIG_PATHS.iter().map(Path::new) {
                if self.resources.exists(path).map_err(element_to_vec)? {
                    configuration_files.push(path);
                }
            }

            match configuration_files.len() {
                0 => {}
                1 => {
                    self.configuration = self
                        .read_configuration(configuration_files.first().unwrap())
                        .map_err(element_to_vec)?;
                }
                _ => {
                    return Err(vec![DarkluaError::multiple_configuration_found(
                        configuration_files.into_iter().map(Path::to_path_buf),
                    )])
                }
            }
        };

        log::trace!(
            "configuration setup in {}",
            configuration_setup_timer.duration_label()
        );
        log::info!(
            "using configuration: {}",
            json5::to_string(&self.configuration).unwrap_or_else(|err| {
                format!("unable to serialize configuration: {}", err.to_string())
            })
        );

        log::trace!("start collecting work");
        let collect_work_timer = Timer::now();

        let collect_work_result: Result<Vec<_>, _> = work_items.collect();
        let mut work_items = collect_work_result.map_err(element_to_vec)?;

        log::trace!("work collected in {}", collect_work_timer.duration_label());

        let mut errors = Vec::new();

        let work_timer = Timer::now();

        'work_loop: while !work_items.is_empty() {
            let work_length = work_items.len();
            log::trace!(
                "working on batch of {} task{}",
                work_length,
                utils::maybe_plural(work_length)
            );

            let mut work_left = Vec::new();

            for work in work_items.into_iter() {
                let work_source_display = work.source().display().to_string();

                match self.do_work(work) {
                    Ok(None) => {
                        log::info!("successfully processed `{}`", work_source_display);
                    }
                    Ok(Some(next_work)) => {
                        log::trace!("work on `{}` has not completed", work_source_display);
                        work_left.push(next_work);
                    }
                    Err(err) => {
                        log::error!("{}", err);
                        errors.push(err);
                        if options.should_fail_fast() {
                            log::info!(
                                "dropping all work because of the fail-fast option is enabled"
                            );
                            break 'work_loop;
                        }
                    }
                }
            }

            if work_left.len() >= work_length {
                return Err(vec![DarkluaError::cyclic_work(work_left)]);
            }

            work_items = work_left;
        }

        log::info!("executed all work in {}", work_timer.duration_label());

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn read_configuration(&self, config: &Path) -> Result<Configuration, DarkluaError> {
        let config_content = self.resources.get(config)?;
        json5::from_str(&config_content).map_err(|err| {
            DarkluaError::invalid_configuration_file(config).context(err.to_string())
        })
    }

    fn do_work(&mut self, work: WorkItem) -> Result<Option<WorkItem>, DarkluaError> {
        let (status, data) = work.extract();
        match status {
            WorkStatus::NotStarted => {
                let source_display = data.source().display();

                let source = data.source();
                let content = self.resources.get(source)?;

                let parser = self.configuration.build_parser();

                log::debug!("beginning work on `{}`", source_display);

                let parser_timer = Timer::now();

                let block = parser
                    .parse(&content)
                    .map_err(|parser_error| DarkluaError::parser_error(source, parser_error))?;

                let parser_time = parser_timer.duration_label();
                log::debug!("parsed `{}` in {}", source_display, parser_time);

                self.apply_rules(data, Progress::new(content, block))
            }
            WorkStatus::InProgress(progress) => self.apply_rules(data, progress),
        }
    }

    fn apply_rules(
        &mut self,
        data: WorkData,
        mut progress: Progress,
    ) -> Result<Option<WorkItem>, DarkluaError> {
        let source_display = data.source().display();

        progress.duration().start();

        for (index, rule) in self
            .configuration
            .rules()
            .enumerate()
            .skip(progress.next_rule())
        {
            let mut context_builder = ContextBuilder::default();
            log::trace!(
                "[{}] apply rule `{}`{}",
                source_display,
                rule.get_name(),
                if rule.has_properties() {
                    format!("{:?}", rule.serialize_to_properties())
                } else {
                    "".to_owned()
                }
            );
            let required_content = rule.require_content(
                // progress.block()
            );

            if !required_content.is_empty() {
                if required_content
                    .iter()
                    .all(|path| self.cache.contains(path))
                {
                    let parser = self.configuration.build_parser();
                    for path in required_content.iter() {
                        let block = self.cache.get_block(path, &parser)?;
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
                        data.with_status(WorkStatus::InProgress(
                            progress
                                .at_rule(index)
                                .with_required_content(required_content),
                        )),
                    ));
                }
            }

            let mut context = context_builder.build();
            rule.process(progress.mutate_block(), &mut context)
                .map_err(|rule_error| {
                    let error = DarkluaError::rule_error(data.source(), rule, index, rule_error);

                    log::trace!(
                        "[{}] rule `{}` errored: {}",
                        source_display,
                        rule.get_name(),
                        error
                    );

                    error
                })?;
        }

        let rule_time = progress.duration().duration_label();
        let total_rules = self.configuration.rules_len();
        log::debug!(
            "{} rule{} applied for `{}` in {}",
            total_rules,
            utils::maybe_plural(total_rules),
            source_display,
            rule_time,
        );

        let generator_timer = Timer::now();

        let lua_code = self
            .configuration
            .generate_lua(progress.block(), progress.content());

        let generator_time = generator_timer.duration_label();
        log::debug!(
            "generated code for `{}` in {}",
            source_display,
            generator_time,
        );

        self.resources.write(data.output(), &lua_code)?;

        self.cache
            .link_source_to_output(data.source(), data.output());

        Ok(None)
    }
}

#[inline]
fn element_to_vec<T>(element: impl Into<T>) -> Vec<T> {
    vec![element.into()]
}
