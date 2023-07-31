use std::path::{Path, PathBuf};

use crate::{
    nodes::{Arguments, Expression, FunctionCall, Prefix},
    process::IdentifierTracker,
    utils,
};

const REQUIRE_FUNCTION_IDENTIFIER: &str = "require";

pub(crate) fn is_require_call(call: &FunctionCall, identifier_tracker: &IdentifierTracker) -> bool {
    if call.get_method().is_some() {
        return false;
    }

    match call.get_prefix() {
        Prefix::Identifier(identifier) => {
            identifier.get_name() == REQUIRE_FUNCTION_IDENTIFIER
                && !identifier_tracker.is_identifier_used(REQUIRE_FUNCTION_IDENTIFIER)
        }
        _ => false,
    }
}

pub(crate) fn match_path_require_call(call: &FunctionCall) -> Option<PathBuf> {
    match call.get_arguments() {
        Arguments::String(string) => Some(string.get_value()),
        Arguments::Tuple(tuple) if tuple.len() == 1 => {
            let expression = tuple.iter_values().next().unwrap();

            match expression {
                Expression::String(string) => Some(string.get_value()),
                _ => None,
            }
        }
        _ => None,
    }
    .map(Path::new)
    .map(utils::normalize_path_with_current_dir)
}
