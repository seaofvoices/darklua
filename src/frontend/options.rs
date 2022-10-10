use std::path::{Path, PathBuf};

use super::configuration::Configuration;

#[derive(Debug)]
pub struct Options {
    input: PathBuf,
    config_path: Option<PathBuf>,
    config: Option<Configuration>,
    output: Option<PathBuf>,
    fail_fast: bool,
}

impl Options {
    pub fn new(input: impl Into<PathBuf>) -> Self {
        Self {
            input: input.into(),
            config_path: None,
            config: None,
            output: None,
            fail_fast: false,
        }
    }

    pub fn with_configuration_at(mut self, config: impl Into<PathBuf>) -> Self {
        self.config_path = Some(config.into());
        self
    }

    pub fn with_configuration(mut self, config: Configuration) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_output(mut self, output: impl Into<PathBuf>) -> Self {
        self.output = Some(output.into());
        self
    }

    pub fn fail_fast(mut self) -> Self {
        self.fail_fast = true;
        self
    }

    pub fn input(&self) -> &Path {
        &self.input
    }

    pub fn output(&self) -> Option<&Path> {
        self.output.as_ref().map(AsRef::as_ref)
    }

    pub fn should_fail_fast(&self) -> bool {
        self.fail_fast
    }

    pub fn configuration_path(&self) -> Option<&Path> {
        self.config_path.as_ref().map(AsRef::as_ref)
    }

    pub fn take_configuration(&mut self) -> Option<Configuration> {
        self.config.take()
    }
}
