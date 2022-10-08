use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Options {
    input: PathBuf,
    config: Option<PathBuf>,
    output: Option<PathBuf>,
    fail_fast: bool,
}

impl Options {
    pub fn new(input: impl Into<PathBuf>) -> Self {
        Self {
            input: input.into(),
            config: None,
            output: None,
            fail_fast: false,
        }
    }

    pub fn with_configuration_at(mut self, config: impl Into<PathBuf>) -> Self {
        self.config = Some(config.into());
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

    pub fn configuration(&self) -> Option<&Path> {
        self.config.as_ref().map(AsRef::as_ref)
    }
}
