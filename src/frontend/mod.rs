mod configuration;
mod error;
mod options;
mod resources;
mod utils;
mod work_cache;
mod work_item;
mod worker;
mod worker_tree;

pub use configuration::{BundleConfiguration, Configuration, GeneratorParameters};
pub use error::{DarkluaError, DarkluaResult};
pub use options::Options;
pub use resources::Resources;
use serde::Serialize;
use work_item::WorkItem;
use worker::Worker;
pub use worker_tree::WorkerTree;

use crate::{
    generator::{DenseLuaGenerator, LuaGenerator},
    nodes::{Block, ReturnStatement},
    process::to_expression,
    utils::normalize_path,
};

/// Convert serializable data into a Lua module
pub fn convert_data(value: impl Serialize) -> Result<String, DarkluaError> {
    let expression = to_expression(&value).map_err(DarkluaError::from)?;

    let block = Block::default()
        .with_last_statement(ReturnStatement::default().with_expression(expression));

    let mut generator = DenseLuaGenerator::default();
    generator.write_block(&block);
    Ok(generator.into_string())
}

pub fn process(resources: &Resources, options: Options) -> DarkluaResult<WorkerTree> {
    let mut worker_tree = WorkerTree::default();

    worker_tree.collect_work(resources, &options)?;
    worker_tree.process(resources, options)?;

    Ok(worker_tree)
}
