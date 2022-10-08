use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};

use elsa::FrozenMap;

use crate::{nodes::Block, Parser};

use super::{utils::Timer, DarkluaError, Resources};

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkStatus {
    NotStarted,
    InProgress(Progress),
}

impl Default for WorkStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

#[derive(Debug, Clone)]
pub struct WorkData {
    source: PathBuf,
    output: PathBuf,
}

impl WorkData {
    pub fn with_status(self, status: WorkStatus) -> WorkItem {
        WorkItem { data: self, status }
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

    #[inline]
    pub fn has_started(&self) -> bool {
        self.status != WorkStatus::NotStarted
    }
}

pub struct WorkCache<'a> {
    resources: &'a Resources,
    input_to_block: FrozenMap<PathBuf, Box<Block>>,
    input_to_output: HashMap<PathBuf, PathBuf>,
}

impl<'a> Clone for WorkCache<'a> {
    fn clone(&self) -> Self {
        Self {
            resources: &self.resources,
            input_to_block: Default::default(),
            input_to_output: self.input_to_output.clone(),
        }
    }
}

impl<'a> fmt::Debug for WorkCache<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkCache")
            .field("resources", &self.resources)
            .field("input_to_output", &self.input_to_output)
            .finish_non_exhaustive()
    }
}

impl<'a> WorkCache<'a> {
    pub fn new(resources: &'a Resources) -> Self {
        Self {
            resources,
            input_to_block: Default::default(),
            input_to_output: Default::default(),
        }
    }

    pub fn link_source_to_output(
        &mut self,
        source: impl Into<PathBuf>,
        output: impl Into<PathBuf>,
    ) {
        self.input_to_output.insert(source.into(), output.into());
    }

    pub fn contains(&self, source: impl AsRef<Path>) -> bool {
        self.input_to_output.contains_key(source.as_ref())
    }

    pub fn get_block(
        &self,
        source: impl AsRef<Path>,
        parser: &Parser,
    ) -> Result<&Block, DarkluaError> {
        let source = source.as_ref();
        if let Some(block) = self.input_to_block.get(source) {
            log::trace!("found cached block for `{}`", source.display());
            Ok(block)
        } else {
            log::trace!("caching block for `{}`", source.display());
            let block = self.read_block(source, parser)?;
            Ok(self
                .input_to_block
                .insert(source.to_path_buf(), Box::new(block)))
        }
    }

    fn read_block(&self, source: &Path, parser: &Parser) -> Result<Block, DarkluaError> {
        if let Some(output_path) = self.input_to_output.get(source) {
            let content = self.resources.get(output_path)?;
            parser.parse(&content).map_err(|parser_error| {
                DarkluaError::parser_error(output_path, parser_error)
                    .context("parsing an already generated file")
            })
        } else {
            Err(DarkluaError::uncached_work(source))
        }
    }
}
