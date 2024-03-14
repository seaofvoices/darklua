use std::path::{Path, PathBuf};

use crate::{nodes::Block, utils::Timer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Progress {
    block: Block,
    next_rule: usize,
    required: Vec<PathBuf>,
    duration: Timer,
}

impl Progress {
    pub fn new(block: Block) -> Self {
        Self {
            block,
            next_rule: 0,
            required: Vec::new(),
            duration: Timer::now(),
        }
    }

    pub fn with_content(self, content: String) -> WorkProgress {
        WorkProgress {
            content,
            work: self,
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

    pub fn duration(&mut self) -> &mut Timer {
        &mut self.duration
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkProgress {
    content: String,
    work: Progress,
}

impl WorkProgress {
    pub fn new(content: String, block: Block) -> Self {
        Self {
            content,
            work: Progress::new(block),
        }
    }

    pub fn required_content(&self) -> impl Iterator<Item = &Path> {
        self.work.required.iter().map(AsRef::as_ref)
    }

    pub fn extract(self) -> (String, Progress) {
        (self.content, self.work)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorkStatus {
    NotStarted,
    InProgress(Box<WorkProgress>),
}

impl Default for WorkStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

impl From<WorkProgress> for WorkStatus {
    fn from(progress: WorkProgress) -> Self {
        Self::InProgress(Box::new(progress))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WorkData {
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
pub(crate) struct WorkItem {
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

    pub fn get_created_file_path(&self) -> Option<PathBuf> {
        if self.data.source == self.data.output {
            None
        } else {
            Some(self.data.output.to_path_buf())
        }
    }

    pub fn total_required_content(&self) -> usize {
        match &self.status {
            WorkStatus::NotStarted => 0,
            WorkStatus::InProgress(progress) => progress.work.required.len(),
        }
    }
}
