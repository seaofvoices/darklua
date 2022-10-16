use crate::DarkluaError;

use super::resources::ResourceError;

#[derive(Debug, Clone)]
pub struct ProcessResult {
    success_count: usize,
    errors: Vec<DarkluaError>,
}

impl ProcessResult {
    pub fn new(success_count: usize, errors: Vec<DarkluaError>) -> Self {
        Self {
            success_count,
            errors,
        }
    }

    pub fn success_count(&self) -> usize {
        self.success_count
    }

    pub fn result(self) -> Result<(), Vec<DarkluaError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }
}

impl From<ResourceError> for ProcessResult {
    fn from(error: ResourceError) -> Self {
        Self {
            success_count: 0,
            errors: vec![error.into()],
        }
    }
}

impl From<DarkluaError> for ProcessResult {
    fn from(error: DarkluaError) -> Self {
        Self {
            success_count: 0,
            errors: vec![error],
        }
    }
}

impl From<Vec<DarkluaError>> for ProcessResult {
    fn from(errors: Vec<DarkluaError>) -> Self {
        Self {
            success_count: 0,
            errors,
        }
    }
}

impl Into<Result<ProcessResult, ProcessResult>> for ProcessResult {
    fn into(self) -> Result<ProcessResult, ProcessResult> {
        if self.errors.is_empty() {
            Ok(self)
        } else {
            Err(self)
        }
    }
}
