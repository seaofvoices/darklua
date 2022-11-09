use std::path::{Path, PathBuf};

use crate::nodes::Block;

use super::utils::Timer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    content: String,
    block: Block,
    next_rule: usize,
    required: Vec<PathBuf>,
    duration: Timer,
}

impl Progress {
    pub fn new(content: String, block: Block) -> Self {
        Self {
            content,
            block,
            next_rule: 0,
            required: Vec::new(),
            duration: Timer::now(),
        }
    }

    pub fn at_rule(mut self, rule_index: usize) -> Self {
        self.next_rule = rule_index;
        self
    }

    pub fn with_required_content(mut self, required_content: Vec<PathBuf>) -> Self {
        self.required = required_content;
        self
    }

    pub fn next_rule(&self) -> usize {
        self.next_rule
    }

    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn duration(&mut self) -> &mut Timer {
        &mut self.duration
    }

    pub fn required_content(&self) -> impl Iterator<Item = &Path> {
        self.required.iter().map(AsRef::as_ref)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkStatus {
    NotStarted,
    InProgress(Box<Progress>),
}

impl Default for WorkStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

impl From<Progress> for WorkStatus {
    fn from(progress: Progress) -> Self {
        Self::InProgress(Box::new(progress))
    }
}

#[derive(Debug, Clone)]
pub struct WorkData {
    source: PathBuf,
    output: PathBuf,
}

impl WorkData {
    pub fn with_status(self, status: impl Into<WorkStatus>) -> WorkItem {
        WorkItem {
            data: self,
            status: status.into(),
        }
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn output(&self) -> &Path {
        &self.output
    }
}

#[derive(Debug, Clone)]
pub struct WorkItem {
    data: WorkData,
    status: WorkStatus,
}

impl WorkItem {
    pub fn new(source: impl Into<PathBuf>, output: impl Into<PathBuf>) -> Self {
        Self {
            data: WorkData {
                source: source.into(),
                output: output.into(),
            },
            status: Default::default(),
        }
    }

    pub fn new_in_place(source: impl Into<PathBuf>) -> Self {
        let source = source.into();
        Self::new(source.clone(), source)
    }

    pub fn extract(self) -> (WorkStatus, WorkData) {
        (self.status, self.data)
    }

    pub fn source(&self) -> &Path {
        &self.data.source
    }

    pub fn total_required_content(&self) -> usize {
        match &self.status {
            WorkStatus::NotStarted => 0,
            WorkStatus::InProgress(progress) => progress.required.len(),
        }
    }
}
