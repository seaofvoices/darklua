use std::path::PathBuf;

use crate::DarkluaError;

use super::resources::ResourceError;

#[derive(Debug, Clone)]
pub struct ProcessResult {
    success_count: usize,
    created_files: Vec<PathBuf>,
    errors: Vec<DarkluaError>,
}

impl ProcessResult {
    pub fn new(
        success_count: usize,
        created_files: Vec<PathBuf>,
        errors: Vec<DarkluaError>,
    ) -> Self {
        Self {
            success_count,
            created_files,
            errors,
        }
    }

    pub fn success_count(&self) -> usize {
        self.success_count
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn into_created_files(self) -> impl Iterator<Item = PathBuf> {
        self.created_files.into_iter()
    }

    pub fn has_errored(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> impl Iterator<Item = &DarkluaError> {
        self.errors.iter()
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
            created_files: Vec::new(),
            errors: vec![error.into()],
        }
    }
}

impl From<DarkluaError> for ProcessResult {
    fn from(error: DarkluaError) -> Self {
        Self {
            success_count: 0,
            created_files: Vec::new(),
            errors: vec![error],
        }
    }
}

impl From<Vec<DarkluaError>> for ProcessResult {
    fn from(errors: Vec<DarkluaError>) -> Self {
        Self {
            success_count: 0,
            created_files: Vec::new(),
            errors,
        }
    }
}

impl From<ProcessResult> for Result<ProcessResult, ProcessResult> {
    fn from(process_result: ProcessResult) -> Self {
        if process_result.errors.is_empty() {
            Ok(process_result)
        } else {
            Err(process_result)
        }
    }
}
