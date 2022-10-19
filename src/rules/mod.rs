//! A module that contains the different rules that mutates a Lua block.

mod call_parens;
mod compute_expression;
mod configuration_error;
mod convert_index_to_field;
mod empty_do;
mod filter_early_return;
mod group_local;
mod inject_value;
mod method_def;
mod no_local_function;
mod remove_comments;
mod remove_nil_declarations;
mod remove_spaces;
mod rename_variables;
mod rule_property;
mod unused_if_branch;
mod unused_while;

pub use call_parens::*;
pub use compute_expression::*;
pub use configuration_error::RuleConfigurationError;
pub use convert_index_to_field::*;
pub use empty_do::*;
pub use filter_early_return::*;
pub use group_local::*;
pub use inject_value::*;
pub use method_def::*;
pub use no_local_function::*;
pub use remove_comments::*;
pub use remove_nil_declarations::*;
pub use remove_spaces::*;
pub use rename_variables::*;
pub use rule_property::*;
pub use unused_if_branch::*;
pub use unused_while::*;

use crate::nodes::Block;

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct ContextBuilder<'a> {
    path: PathBuf,
    blocks: HashMap<PathBuf, &'a Block>,
}

impl<'a> ContextBuilder<'a> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            blocks: Default::default(),
        }
    }

    pub fn build(self) -> Context<'a> {
        Context {
            path: self.path,
            blocks: self.blocks,
        }
    }

    pub fn insert_block<'b: 'a>(&mut self, path: impl Into<PathBuf>, block: &'b Block) {
        self.blocks.insert(path.into(), block);
    }
}

/// The intent of this struct is to hold data shared across all rules applied to a file.
#[derive(Debug, Clone, Default)]
pub struct Context<'a> {
    path: PathBuf,
    blocks: HashMap<PathBuf, &'a Block>,
}

impl<'a> Context<'a> {
    pub fn block(&self, path: impl AsRef<Path>) -> Option<&Block> {
        self.blocks.get(path.as_ref()).copied()
    }

    pub fn current_path(&self) -> &Path {
        self.path.as_ref()
    }
}

pub type RuleProcessResult = Result<(), String>;

/// Defines an interface that will be used to mutate blocks and how to serialize and deserialize
/// the rule configuration.
pub trait Rule: RuleConfiguration {
    /// This method should mutate the given block to apply the rule
    fn process(&self, block: &mut Block, context: &mut Context) -> RuleProcessResult;

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
    fn flawless_process(&self, block: &mut Block, context: &mut Context);
}

impl<T: FlawlessRule + RuleConfiguration> Rule for T {
    fn process(&self, block: &mut Block, context: &mut Context) -> RuleProcessResult {
        self.flawless_process(block, context);
        Ok(())
    }
}

/// A function to get the default rule stack for darklua. All the rules here must preserve all the
/// functionalities of the original code after being applied. They must guarantee that the
/// processed block will work as much as the original one.
pub fn get_default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(RemoveSpaces::default()),
        Box::new(RemoveComments::default()),
        Box::new(ComputeExpression::default()),
        Box::new(RemoveUnusedIfBranch::default()),
        Box::new(RemoveUnusedWhile::default()),
        Box::new(FilterAfterEarlyReturn::default()),
        Box::new(RemoveEmptyDo::default()),
        Box::new(RemoveMethodDefinition::default()),
        Box::new(ConvertIndexToField::default()),
        Box::new(RemoveNilDeclaration::default()),
        Box::new(RenameVariables::default()),
        Box::new(RemoveFunctionCallParens::default()),
    ]
}

pub fn get_all_rule_names() -> Vec<&'static str> {
    vec![
        COMPUTE_EXPRESSIONS_RULE_NAME,
        CONVERT_INDEX_TO_FIELD_RULE_NAME,
        CONVERT_LOCAL_FUNCTION_TO_ASSIGN_RULE_NAME,
        FILTER_AFTER_EARLY_RETURN_RULE_NAME,
        GROUP_LOCAL_ASSIGNMENT_RULE_NAME,
        INJECT_GLOBAL_VALUE_RULE_NAME,
        REMOVE_COMMENTS_RULE_NAME,
        REMOVE_EMPTY_DO_RULE_NAME,
        REMOVE_FUNCTION_CALL_PARENS_RULE_NAME,
        REMOVE_METHOD_DEFINITION_RULE_NAME,
        REMOVE_NIL_DECLARATION_RULE_NAME,
        REMOVE_SPACES_RULE_NAME,
        REMOVE_UNUSED_IF_BRANCH_RULE_NAME,
        REMOVE_UNUSED_WHILE_RULE_NAME,
        RENAME_VARIABLES_RULE_NAME,
    ]
}

impl FromStr for Box<dyn Rule> {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let rule: Box<dyn Rule> = match string {
            COMPUTE_EXPRESSIONS_RULE_NAME => Box::new(ComputeExpression::default()),
            CONVERT_INDEX_TO_FIELD_RULE_NAME => Box::new(ConvertIndexToField::default()),
            CONVERT_LOCAL_FUNCTION_TO_ASSIGN_RULE_NAME => {
                Box::new(ConvertLocalFunctionToAssign::default())
            }
            FILTER_AFTER_EARLY_RETURN_RULE_NAME => Box::new(FilterAfterEarlyReturn::default()),
            GROUP_LOCAL_ASSIGNMENT_RULE_NAME => Box::new(GroupLocalAssignment::default()),
            INJECT_GLOBAL_VALUE_RULE_NAME => Box::new(InjectGlobalValue::default()),
            REMOVE_COMMENTS_RULE_NAME => Box::new(RemoveComments::default()),
            REMOVE_EMPTY_DO_RULE_NAME => Box::new(RemoveEmptyDo::default()),
            REMOVE_FUNCTION_CALL_PARENS_RULE_NAME => Box::new(RemoveFunctionCallParens::default()),
            REMOVE_METHOD_DEFINITION_RULE_NAME => Box::new(RemoveMethodDefinition::default()),
            REMOVE_NIL_DECLARATION_RULE_NAME => Box::new(RemoveNilDeclaration::default()),
            REMOVE_SPACES_RULE_NAME => Box::new(RemoveSpaces::default()),
            REMOVE_UNUSED_IF_BRANCH_RULE_NAME => Box::new(RemoveUnusedIfBranch::default()),
            REMOVE_UNUSED_WHILE_RULE_NAME => Box::new(RemoveUnusedWhile::default()),
            RENAME_VARIABLES_RULE_NAME => Box::new(RenameVariables::default()),
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
