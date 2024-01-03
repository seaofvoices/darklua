use std::path::Path;

use super::{
    configuration::Configuration, resources::Resources, work_item::WorkItem, DarkluaError,
    DarkluaResult, Options, ProcessResult,
};

use crate::{frontend::async_worker::AsyncWorker, utils::Timer, GeneratorParameters};

const DEFAULT_CONFIG_PATHS: [&str; 2] = [".darklua.json", ".darklua.json5"];

#[derive(Debug)]
pub(crate) struct Worker<'a> {
    resources: &'a Resources,
    configuration: Configuration,
}

impl<'a> Worker<'a> {
    pub fn new(resources: &'a Resources) -> Self {
        Self {
            resources,
            configuration: Configuration::default(),
        }
    }

    pub async fn process(
        mut self,
        work_items: impl Iterator<Item = Result<WorkItem, DarkluaError>>,
        mut options: Options,
    ) -> Result<ProcessResult, ProcessResult> {
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
            if self.resources.exists(config).await? {
                self.configuration = self.read_configuration(config).await?;
                log::info!("using configuration file `{}`", config.display());
            } else {
                return Err(DarkluaError::resource_not_found(config)
                    .context("expected to find configuration file as provided by the options")
                    .into());
            }
        } else {
            let mut configuration_files = Vec::new();
            for path in DEFAULT_CONFIG_PATHS.iter().map(Path::new) {
                if self.resources.exists(path).await? {
                    configuration_files.push(path);
                }
            }

            match configuration_files.len() {
                0 => {
                    log::info!("using default configuration");
                }
                1 => {
                    let configuration_file_path = configuration_files.first().unwrap();
                    self.configuration = self
                        .read_configuration(configuration_file_path)
                        .await
                        .map_err(element_to_vec)?;
                    log::info!(
                        "using configuration file `{}`",
                        configuration_file_path.display()
                    );
                }
                _ => {
                    return Err(DarkluaError::multiple_configuration_found(
                        configuration_files.into_iter().map(Path::to_path_buf),
                    )
                    .into())
                }
            }
        };

        if let Some(generator) = options.generator_override() {
            log::trace!(
                "override with {} generator",
                match generator {
                    GeneratorParameters::RetainLines => "`retain_lines`".to_owned(),
                    GeneratorParameters::Dense { column_span } =>
                        format!("dense ({})", column_span),
                    GeneratorParameters::Readable { column_span } =>
                        format!("readable ({})", column_span),
                }
            );
            self.configuration = self.configuration.with_generator(generator.clone());
        }

        log::trace!(
            "configuration setup in {}",
            configuration_setup_timer.duration_label()
        );
        log::debug!(
            "using configuration: {}",
            json5::to_string(&self.configuration).unwrap_or_else(|err| {
                format!("? (unable to serialize configuration: {})", err)
            })
        );

        log::trace!("start collecting work");
        let collect_work_timer = Timer::now();

        let collect_work_result: Result<Vec<_>, _> = work_items.collect();
        let work_items = collect_work_result.map_err(element_to_vec)?;

        log::trace!("work collected in {}", collect_work_timer.duration_label());

        let work_timer = Timer::now();

        let process_result = AsyncWorker::new(self.resources.clone(), self.configuration)
            .process(work_items, options.should_fail_fast())
            .await;

        log::info!("executed work in {}", work_timer.duration_label());

        // ProcessResult::new(success_count, errors).into()
        process_result
    }

    async fn read_configuration(&self, config: &Path) -> DarkluaResult<Configuration> {
        let config_content = self.resources.get(config).await?;
        json5::from_str(&config_content)
            .map_err(|err| {
                DarkluaError::invalid_configuration_file(config).context(err.to_string())
            })
            .map(|configuration: Configuration| {
                configuration.with_location({
                    config.parent().unwrap_or_else(|| {
                        log::warn!(
                            "unexpected configuration path `{}` (unable to extract parent path)",
                            config.display()
                        );
                        config
                    })
                })
            })
    }
}

#[inline]
fn element_to_vec<T>(element: impl Into<T>) -> Vec<T> {
    vec![element.into()]
}
