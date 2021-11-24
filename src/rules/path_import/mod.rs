mod resolver;
mod roblox;

use std::mem;
use std::path::{Path, PathBuf};

use crate::nodes::{Arguments, Block, Expression, FunctionCall, Prefix};
use crate::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
use crate::rules::{
    verify_required_rule_properties, Context, Rule, RuleConfiguration, RuleConfigurationError,
    RuleProcessResult, RuleProperties, RulePropertyValue,
};

use serde::ser::{Serialize, Serializer};

use resolver::Resolver;

use roblox::{RobloxIndexingStrategy, RobloxLibraryResolver, RobloxLocation, RobloxRootLocation};

#[derive(Debug)]
struct Processor<R: Resolver> {
    resolver: R,
}

impl<R: Resolver> Processor<R> {
    fn new(resolver: R) -> Self {
        Self { resolver }
    }

    fn call_prefix_is_import(&self, call: &FunctionCall) -> bool {
        matches!(call.get_prefix(), Prefix::Identifier(identifier) if identifier.contains("import"))
    }

    fn get_path_from_call(&self, call: &FunctionCall) -> Option<PathBuf> {
        match call.get_arguments() {
            Arguments::Tuple(tuple) => {
                if tuple.len() == 1 {
                    let value = tuple.iter_values().next().unwrap();
                    match value {
                        Expression::String(string) => Some(string.get_value().into()),
                        _ => None, // TODO: report error
                    }
                } else {
                    None // TODO: report error
                }
            }
            Arguments::String(string) => Some(string.get_value().into()),
            Arguments::Table(_) => None, // TODO: report error
        }
    }

    fn resolve_path(&self, path: &Path) -> Option<Expression> {
        self.resolver.resolve(path).map(|expression| {
            FunctionCall::from_name("require")
                .with_argument(expression)
                .into()
        })
    }
}

impl<R: Resolver> NodeProcessor for Processor<R> {
    fn process_expression(&mut self, expression: &mut Expression) {
        // TODO: verify that `import` is not overriden
        match expression {
            Expression::Call(call) if self.call_prefix_is_import(call) => {
                if let Some(path) = self.get_path_from_call(call) {
                    if let Some(mut location) = self.resolve_path(&path) {
                        mem::swap(expression, &mut location);
                    } else {
                        todo!("report error: can't resolve path")
                    }
                }
            }
            _ => {}
        }
    }
}

pub const PATH_IMPORT_RULE_NAME: &str = "path_import";
static VALID_RESOLVERS: [&'static str; 1] = ["roblox_library"];

/// Convert local function statements into local assignements when the function is not recursive.
#[derive(Debug)]
pub struct PathImport {
    resolver_name: String,
}

impl PathImport {
    fn process_with_resolver<R: Resolver>(
        &self,
        block: &mut Block,
        resolver: R,
    ) -> RuleProcessResult {
        let mut processor = Processor::new(resolver);
        DefaultVisitor::visit_block(block, &mut processor);
        Ok(())
    }

    fn build_resolver(&self, context: &Context) -> impl Resolver {
        // match self.resolver_name {
        // TODO
        // }
        RobloxLibraryResolver::new(
            RobloxIndexingStrategy::FindFirstChild,
            PathBuf::new(),
            context.current_file_path.clone(),
            RobloxLocation::from(RobloxRootLocation::Script),
        )
    }
}

impl Rule for PathImport {
    fn process(&self, block: &mut Block, context: &mut Context) -> RuleProcessResult {
        let resolver = self.build_resolver(context);
        self.process_with_resolver(block, resolver)
    }
}

impl RuleConfiguration for PathImport {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_required_rule_properties(&properties, &["resolver"])?;

        for (key, value) in properties {
            match key.as_str() {
                "resolver" => match value {
                    RulePropertyValue::String(resolver) => {
                        if !VALID_RESOLVERS.contains(&resolver.as_str()) {
                            return Err(RuleConfigurationError::UnexpectedValue {
                                property: key,
                                value: resolver,
                                suggestion: Some(format!(
                                    "expected '{}'",
                                    VALID_RESOLVERS.join("', '")
                                )),
                            });
                        }
                        self.resolver_name = resolver;
                    }
                    _ => return Err(RuleConfigurationError::StringExpected(key)),
                },
                _ => return Err(RuleConfigurationError::UnexpectedProperty(key)),
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        PATH_IMPORT_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}

impl Serialize for PathImport {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(PATH_IMPORT_RULE_NAME)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use insta::assert_json_snapshot;

    // fn new_rule() -> PathImport {
    //     // PathImport::default()
    //     // todo!()
    // }

    // #[test]
    // fn serialize_default_rule() {
    //     assert_json_snapshot!("default_path_import", new_rule());
    // }
}
