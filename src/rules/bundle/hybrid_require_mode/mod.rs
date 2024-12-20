use crate::{nodes::Block, process::{NodeVisitor, ScopeVisitor}, rules::{bundle::BundleOptions, require::HybridRequireMode, Context, FlawlessRule, ReplaceReferencedTokens}, utils::Timer};

use super::path_require_mode::RequirePathProcessor;

pub(crate) fn process_block(
    block: &mut Block,
    context: &Context,
    options: &BundleOptions,
    path_require_mode: &HybridRequireMode,
) -> Result<(), String> {
    if options.parser().is_preserving_tokens() {
        log::trace!(
            "replacing token references of {}",
            context.current_path().display()
        );
        let replace_tokens = ReplaceReferencedTokens::default();

        let apply_replace_tokens_timer = Timer::now();

        replace_tokens.flawless_process(block, context);

        log::trace!(
            "replaced token references for `{}` in {}",
            context.current_path().display(),
            apply_replace_tokens_timer.duration_label()
        );
    }

    let mut processor = RequirePathProcessor::new(context, options, path_require_mode);
    ScopeVisitor::visit_block(block, &mut processor);
    processor.apply(block, context)
}