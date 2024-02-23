//! A module that contains the different rules that mutates a Lua block.

mod append_text_comment;
pub mod bundle;
mod call_parens;
mod compute_expression;
mod configuration_error;
mod convert_index_to_field;
mod convert_require;
mod empty_do;
mod filter_early_return;
mod group_local;
mod inject_value;
mod method_def;
mod no_local_function;
mod remove_assertions;
mod remove_call_match;
mod remove_comments;
mod remove_compound_assign;
mod remove_debug_profiling;
mod remove_interpolated_string;
mod remove_nil_declarations;
mod remove_spaces;
mod remove_types;
mod remove_unused_variable;
mod rename_variables;
mod replace_referenced_tokens;
pub(crate) mod require;
mod rule_property;
mod shift_token_line;
mod unused_if_branch;
mod unused_while;

pub use append_text_comment::*;
pub use call_parens::*;
pub use compute_expression::*;
pub use configuration_error::RuleConfigurationError;
pub use convert_index_to_field::*;
pub use convert_require::*;
pub use empty_do::*;
pub use filter_early_return::*;
pub use group_local::*;
pub use inject_value::*;
pub use method_def::*;
pub use no_local_function::*;
pub use remove_assertions::*;
pub use remove_comments::*;
pub use remove_compound_assign::*;
pub use remove_debug_profiling::*;
pub use remove_interpolated_string::*;
pub use remove_nil_declarations::*;
pub use remove_spaces::*;
pub use remove_types::*;
pub use remove_unused_variable::*;
pub use rename_variables::*;
pub(crate) use replace_referenced_tokens::*;
pub use rule_property::*;
pub(crate) use shift_token_line::*;
pub use unused_if_branch::*;
pub use unused_while::*;

use crate::nodes::Block;
use crate::Resources;

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct ContextBuilder<'a, 'resources, 'code> {
    path: PathBuf,
    resources: &'resources Resources,
    original_code: &'code str,
    blocks: HashMap<PathBuf, &'a Block>,
    project_location: Option<PathBuf>,
}

impl<'a, 'resources, 'code> ContextBuilder<'a, 'resources, 'code> {
    pub fn new(
        path: impl Into<PathBuf>,
        resources: &'resources Resources,
        original_code: &'code str,
    ) -> Self {
        Self {
            path: path.into(),
            resources,
            original_code,
            blocks: Default::default(),
            project_location: None,
        }
    }

    pub fn with_project_location(mut self, path: impl Into<PathBuf>) -> Self {
        self.project_location = Some(path.into());
        self
    }

    pub fn build(self) -> Context<'a, 'resources, 'code> {
        Context {
            path: self.path,
            resources: self.resources,
            original_code: self.original_code,
            blocks: self.blocks,
            project_location: self.project_location,
        }
    }

    pub fn insert_block<'block: 'a>(&mut self, path: impl Into<PathBuf>, block: &'block Block) {
        self.blocks.insert(path.into(), block);
    }
}

/// The intent of this struct is to hold data shared across all rules applied to a file.
#[derive(Debug, Clone)]
pub struct Context<'a, 'resources, 'code> {
    path: PathBuf,
    resources: &'resources Resources,
    original_code: &'code str,
    blocks: HashMap<PathBuf, &'a Block>,
    project_location: Option<PathBuf>,
}

impl<'a, 'resources, 'code> Context<'a, 'resources, 'code> {
    pub fn block(&self, path: impl AsRef<Path>) -> Option<&Block> {
        self.blocks.get(path.as_ref()).copied()
    }

    pub fn current_path(&self) -> &Path {
        self.path.as_ref()
    }

    fn resources(&self) -> &Resources {
        self.resources
    }

    fn original_code(&self) -> &str {
        self.original_code
    }

    fn project_location(&self) -> &Path {
        self.project_location.as_deref().unwrap_or_else(|| {
            let source = self.current_path();
            source.parent().unwrap_or_else(|| {
                log::warn!(
                    "unexpected file path `{}` (unable to extract parent path)",
                    source.display()
                );
                source
            })
        })
    }
}

pub type RuleProcessResult = Result<(), String>;

/// Defines an interface that will be used to mutate blocks and how to serialize and deserialize
/// the rule configuration.
pub trait Rule: RuleConfiguration + fmt::Debug {
    /// This method should mutate the given block to apply the rule
    fn process(&self, block: &mut Block, context: &Context) -> RuleProcessResult;

    /// Return the list of paths to Lua files that is necessary to apply this rule. This will load
    /// each AST block from these files into the context object.
    fn require_content(&self, _current_source: &Path, _current_block: &Block) -> Vec<PathBuf> {
        Vec::new()
    }
}

pub trait RuleConfiguration {
    /// The rule deserializer will construct the default rule and then send the properties through
    /// this method to modify the behavior of the rule.
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError>;
    /// This method should return the unique name of the rule.
    fn get_name(&self) -> &'static str;
    /// For implementing the serialize trait on the Rule trait, this method should return all
    /// properties that differs from their default value.
    fn serialize_to_properties(&self) -> RuleProperties;
    /// Returns `true` if the rule has at least one property.
    fn has_properties(&self) -> bool {
        !self.serialize_to_properties().is_empty()
    }
}

pub trait FlawlessRule {
    fn flawless_process(&self, block: &mut Block, context: &Context);
}

impl<T: FlawlessRule + RuleConfiguration + fmt::Debug> Rule for T {
    fn process(&self, block: &mut Block, context: &Context) -> RuleProcessResult {
        self.flawless_process(block, context);
        Ok(())
    }
}

/// A function to get the default rule stack for darklua. All the rules here must preserve all the
/// functionalities of the original code after being applied. They must guarantee that the
/// processed block will work as much as the original one.
pub fn get_default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::<RemoveSpaces>::default(),
        Box::<RemoveComments>::default(),
        Box::<ComputeExpression>::default(),
        Box::<RemoveUnusedIfBranch>::default(),
        Box::<RemoveUnusedWhile>::default(),
        Box::<FilterAfterEarlyReturn>::default(),
        Box::<RemoveEmptyDo>::default(),
        Box::<RemoveUnusedVariable>::default(),
        Box::<RemoveMethodDefinition>::default(),
        Box::<ConvertIndexToField>::default(),
        Box::<RemoveNilDeclaration>::default(),
        Box::<RenameVariables>::default(),
        Box::<RemoveFunctionCallParens>::default(),
    ]
}

pub fn get_all_rule_names() -> Vec<&'static str> {
    vec![
        APPEND_TEXT_COMMENT_RULE_NAME,
        COMPUTE_EXPRESSIONS_RULE_NAME,
        CONVERT_INDEX_TO_FIELD_RULE_NAME,
        CONVERT_LOCAL_FUNCTION_TO_ASSIGN_RULE_NAME,
        CONVERT_REQUIRE_RULE_NAME,
        FILTER_AFTER_EARLY_RETURN_RULE_NAME,
        GROUP_LOCAL_ASSIGNMENT_RULE_NAME,
        INJECT_GLOBAL_VALUE_RULE_NAME,
        REMOVE_ASSERTIONS_RULE_NAME,
        REMOVE_COMMENTS_RULE_NAME,
        REMOVE_COMPOUND_ASSIGNMENT_RULE_NAME,
        REMOVE_DEBUG_PROFILING_RULE_NAME,
        REMOVE_EMPTY_DO_RULE_NAME,
        REMOVE_FUNCTION_CALL_PARENS_RULE_NAME,
        REMOVE_INTERPOLATED_STRING_RULE_NAME,
        REMOVE_METHOD_DEFINITION_RULE_NAME,
        REMOVE_NIL_DECLARATION_RULE_NAME,
        REMOVE_SPACES_RULE_NAME,
        REMOVE_TYPES_RULE_NAME,
        REMOVE_UNUSED_IF_BRANCH_RULE_NAME,
        REMOVE_UNUSED_VARIABLE_RULE_NAME,
        REMOVE_UNUSED_WHILE_RULE_NAME,
        RENAME_VARIABLES_RULE_NAME,
    ]
}

impl FromStr for Box<dyn Rule> {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let rule: Box<dyn Rule> = match string {
            APPEND_TEXT_COMMENT_RULE_NAME => Box::<AppendTextComment>::default(),
            COMPUTE_EXPRESSIONS_RULE_NAME => Box::<ComputeExpression>::default(),
            CONVERT_INDEX_TO_FIELD_RULE_NAME => Box::<ConvertIndexToField>::default(),
            CONVERT_LOCAL_FUNCTION_TO_ASSIGN_RULE_NAME => {
                Box::<ConvertLocalFunctionToAssign>::default()
            }
            CONVERT_REQUIRE_RULE_NAME => Box::<ConvertRequire>::default(),
            FILTER_AFTER_EARLY_RETURN_RULE_NAME => Box::<FilterAfterEarlyReturn>::default(),
            GROUP_LOCAL_ASSIGNMENT_RULE_NAME => Box::<GroupLocalAssignment>::default(),
            INJECT_GLOBAL_VALUE_RULE_NAME => Box::<InjectGlobalValue>::default(),
            REMOVE_ASSERTIONS_RULE_NAME => Box::<RemoveAssertions>::default(),
            REMOVE_COMMENTS_RULE_NAME => Box::<RemoveComments>::default(),
            REMOVE_COMPOUND_ASSIGNMENT_RULE_NAME => Box::<RemoveCompoundAssignment>::default(),
            REMOVE_DEBUG_PROFILING_RULE_NAME => Box::<RemoveDebugProfiling>::default(),
            REMOVE_EMPTY_DO_RULE_NAME => Box::<RemoveEmptyDo>::default(),
            REMOVE_FUNCTION_CALL_PARENS_RULE_NAME => Box::<RemoveFunctionCallParens>::default(),
            REMOVE_INTERPOLATED_STRING_RULE_NAME => Box::<RemoveInterpolatedString>::default(),
            REMOVE_METHOD_DEFINITION_RULE_NAME => Box::<RemoveMethodDefinition>::default(),
            REMOVE_NIL_DECLARATION_RULE_NAME => Box::<RemoveNilDeclaration>::default(),
            REMOVE_SPACES_RULE_NAME => Box::<RemoveSpaces>::default(),
            REMOVE_TYPES_RULE_NAME => Box::<RemoveTypes>::default(),
            REMOVE_UNUSED_IF_BRANCH_RULE_NAME => Box::<RemoveUnusedIfBranch>::default(),
            REMOVE_UNUSED_VARIABLE_RULE_NAME => Box::<RemoveUnusedVariable>::default(),
            REMOVE_UNUSED_WHILE_RULE_NAME => Box::<RemoveUnusedWhile>::default(),
            RENAME_VARIABLES_RULE_NAME => Box::<RenameVariables>::default(),
            _ => return Err(format!("invalid rule name: {}", string)),
        };

        Ok(rule)
    }
}

impl Serialize for dyn Rule {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let properties = self.serialize_to_properties();
        let property_count = properties.len();
        let rule_name = self.get_name();

        if property_count == 0 {
            serializer.serialize_str(rule_name)
        } else {
            let mut map = serializer.serialize_map(Some(property_count + 1))?;

            map.serialize_entry("rule", rule_name)?;

            let mut ordered: Vec<(String, RulePropertyValue)> = properties.into_iter().collect();

            ordered.sort_by(|a, b| a.0.cmp(&b.0));

            for (key, value) in ordered {
                map.serialize_entry(&key, &value)?;
            }

            map.end()
        }
    }
}

impl<'de> Deserialize<'de> for Box<dyn Rule> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Box<dyn Rule>, D::Error> {
        struct StringOrStruct;

        impl<'de> Visitor<'de> for StringOrStruct {
            type Value = Box<dyn Rule>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("rule name or rule object")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let mut rule: Self::Value = FromStr::from_str(value).map_err(de::Error::custom)?;

                rule.configure(RuleProperties::new())
                    .map_err(de::Error::custom)?;

                Ok(rule)
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut rule_name = None;
                let mut properties = HashMap::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "rule" => {
                            if rule_name.is_none() {
                                rule_name.replace(map.next_value::<String>()?);
                            } else {
                                return Err(de::Error::duplicate_field("rule"));
                            }
                        }
                        property => {
                            let value = map.next_value::<RulePropertyValue>()?;

                            if properties.insert(property.to_owned(), value).is_some() {
                                return Err(de::Error::custom(format!(
                                    "duplicate field {} in rule object",
                                    property
                                )));
                            }
                        }
                    }
                }

                if let Some(rule_name) = rule_name {
                    let mut rule: Self::Value =
                        FromStr::from_str(&rule_name).map_err(de::Error::custom)?;

                    rule.configure(properties).map_err(de::Error::custom)?;

                    Ok(rule)
                } else {
                    Err(de::Error::missing_field("rule"))
                }
            }
        }

        deserializer.deserialize_any(StringOrStruct)
    }
}

fn verify_no_rule_properties(properties: &RuleProperties) -> Result<(), RuleConfigurationError> {
    if let Some((key, _value)) = properties.iter().next() {
        return Err(RuleConfigurationError::UnexpectedProperty(key.to_owned()));
    }
    Ok(())
}

fn verify_required_properties(
    properties: &RuleProperties,
    names: &[&str],
) -> Result<(), RuleConfigurationError> {
    for name in names.iter() {
        if !properties.contains_key(*name) {
            return Err(RuleConfigurationError::MissingProperty(name.to_string()));
        }
    }
    Ok(())
}

fn verify_required_any_properties(
    properties: &RuleProperties,
    names: &[&str],
) -> Result<(), RuleConfigurationError> {
    if names.iter().any(|name| properties.contains_key(*name)) {
        Ok(())
    } else {
        Err(RuleConfigurationError::MissingAnyProperty(
            names.iter().map(ToString::to_string).collect(),
        ))
    }
}

fn verify_property_collisions(
    properties: &RuleProperties,
    names: &[&str],
) -> Result<(), RuleConfigurationError> {
    let mut exists = false;
    for name in names.iter() {
        if properties.contains_key(*name) {
            if exists {
                return Err(RuleConfigurationError::PropertyCollision(
                    names.iter().map(ToString::to_string).collect(),
                ));
            } else {
                exists = true;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use insta::assert_json_snapshot;

    #[test]
    fn snapshot_default_rules() {
        let rules = get_default_rules();

        assert_json_snapshot!("default_rules", rules);
    }

    #[test]
    fn snapshot_all_rules() {
        let rule_names = get_all_rule_names();

        assert_json_snapshot!("all_rule_names", rule_names);
    }

    #[test]
    fn verify_no_rule_properties_is_ok_when_empty() {
        let empty_properties = RuleProperties::default();

        assert_eq!(verify_no_rule_properties(&empty_properties), Ok(()));
    }

    #[test]
    fn verify_no_rule_properties_is_unexpected_rule_err() {
        let mut properties = RuleProperties::default();
        let some_rule_name = "rule name";
        properties.insert(some_rule_name.to_owned(), RulePropertyValue::None);

        assert_eq!(
            verify_no_rule_properties(&properties),
            Err(RuleConfigurationError::UnexpectedProperty(
                some_rule_name.to_owned()
            ))
        );
    }

    #[test]
    fn get_all_rule_names_are_deserializable() {
        for name in get_all_rule_names() {
            let rule: Result<Box<dyn Rule>, _> = name.parse();
            assert!(rule.is_ok(), "unable to deserialize `{}`", name);
        }
    }

    #[test]
    fn get_all_rule_names_are_serializable() {
        for name in get_all_rule_names() {
            let rule: Box<dyn Rule> = name
                .parse()
                .unwrap_or_else(|_| panic!("unable to deserialize `{}`", name));
            assert!(json5::to_string(&rule).is_ok());
        }
    }
}
