use crate::{
    nodes::Block,
    rules::{
        bundle::BundleOptions, require::HybridRequireMode, Context,
    },
};

use super::process_block_generic;

pub(crate) fn process_block(
    block: &mut Block,
    context: &Context,
    options: &BundleOptions,
    hybrid_require_mode: &HybridRequireMode,
) -> Result<(), String> {
    process_block_generic(block, context, options, hybrid_require_mode) 
}
