use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};

use elsa::FrozenMap;

use crate::{nodes::Block, DarkluaError, Parser, Resources};

use super::DarkluaResult;

pub struct WorkCache {
    resources: Resources,
    input_to_block: FrozenMap<PathBuf, Box<Block>>,
    input_to_output: HashMap<PathBuf, PathBuf>,
}

impl Clone for WorkCache {
    fn clone(&self) -> Self {
        Self {
            resources: self.resources.clone(),
            input_to_block: Default::default(),
            input_to_output: self.input_to_output.clone(),
        }
    }
}

impl fmt::Debug for WorkCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkCache")
            .field("resources", &self.resources)
            .field("input_to_output", &self.input_to_output)
            .finish_non_exhaustive()
    }
}

impl WorkCache {
    pub fn new(resources: &Resources) -> Self {
        Self {
            resources: resources.clone(),
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

    pub async fn get_block(
        &self,
        source: impl AsRef<Path>,
        parser: &Parser,
    ) -> DarkluaResult<&Block> {
        let source = source.as_ref();
        if let Some(block) = self.input_to_block.get(source) {
            log::trace!("found cached block for `{}`", source.display());
            Ok(block)
        } else {
            log::trace!("caching block for `{}`", source.display());
            let block = self.read_block(source, parser).await?;
            Ok(self
                .input_to_block
                .insert(source.to_path_buf(), Box::new(block)))
        }
    }

    async fn read_block(&self, source: &Path, parser: &Parser) -> DarkluaResult<Block> {
        if let Some(output_path) = self.input_to_output.get(source) {
            let content = self.resources.get(output_path).await?;
            parser.parse(&content).map_err(|parser_error| {
                DarkluaError::parser_error(output_path, parser_error)
                    .context("parsing an already generated file")
            })
        } else {
            Err(DarkluaError::uncached_work(source))
        }
    }
}
