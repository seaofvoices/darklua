mod globals;
mod permutator;
mod rename_processor;

use permutator::Permutator;
use rename_processor::RenameProcessor;

use crate::nodes::Block;
use crate::process::{NodeVisitor, ScopeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
    RulePropertyValue,
};

use std::collections::HashSet;
use std::iter::FromIterator;

pub const RENAME_VARIABLES_RULE_NAME: &'static str = "rename_variables";

/// Rename all identifiers to small and meaningless names.
#[derive(Debug, PartialEq, Eq)]
pub struct RenameVariables {
    globals: Vec<String>,
}

fn is_valid_identifier(identifier: &str) -> bool {
    identifier.len() > 0
        && identifier.is_ascii()
        && identifier
            .char_indices()
            .all(|(i, c)| c.is_alphabetic() || c == '_' || (c.is_ascii_digit() && i > 0))
}

impl RenameVariables {
    pub fn new<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self {
            globals: Vec::from_iter(iter),
        }
    }

    fn set_globals(&mut self, list: Vec<String>) -> Result<(), RuleConfigurationError> {
        for value in list {
            match value.as_str() {
                "$default" => self
                    .globals
                    .extend(globals::DEFAULT.to_vec().into_iter().map(String::from)),
                "$roblox" => self
                    .globals
                    .extend(globals::ROBLOX.to_vec().into_iter().map(String::from)),
                identifier if !is_valid_identifier(identifier) => {
                    return Err(RuleConfigurationError::StringExpected("".to_owned()))
                }
                _ => self.globals.push(value),
            }
        }

        Ok(())
    }

    fn normalize_globals(&self) -> Vec<String> {
        let mut globals_set: HashSet<String> = HashSet::from_iter(self.globals.iter().cloned());

        let mut result = Vec::new();

        if globals::DEFAULT
            .iter()
            .all(|identifier| globals_set.contains(*identifier))
        {
            globals::DEFAULT.iter().for_each(|identifier| {
                globals_set.remove(*identifier);
            });
            result.push("$default".to_owned());
        }

        if globals::ROBLOX
            .iter()
            .all(|identifier| globals_set.contains(*identifier))
        {
            globals::ROBLOX.iter().for_each(|identifier| {
                globals_set.remove(*identifier);
            });
            result.push("$roblox".to_owned());
        }

        result.extend(globals_set.into_iter());
        result.sort();
        result
    }
}

impl Default for RenameVariables {
    fn default() -> Self {
        Self::new(globals::DEFAULT.iter().map(|string| (*string).to_owned()))
    }
}

impl FlawlessRule for RenameVariables {
    fn flawless_process(&self, block: &mut Block, _: &mut Context) {
        let mut processor = RenameProcessor::new(self.globals.clone().into_iter());
        ScopeVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for RenameVariables {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        for (key, value) in properties {
            match key.as_str() {
                "globals" => match value {
                    RulePropertyValue::StringList(globals) => self.set_globals(globals)?,
                    _ => return Err(RuleConfigurationError::StringListExpected(key)),
                },
                _ => return Err(RuleConfigurationError::UnexpectedProperty(key)),
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        RENAME_VARIABLES_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        if self == &Self::default() {
            RuleProperties::new()
        } else {
            let mut properties = RuleProperties::new();

            properties.insert(
                "globals".to_owned(),
                RulePropertyValue::StringList(self.normalize_globals()),
            );

            properties
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rules::Rule;

    use insta::assert_json_snapshot;
    use std::iter::empty;

    fn new_rule() -> Box<dyn Rule> {
        Box::new(RenameVariables::default())
    }

    #[test]
    fn serialize_default_rule() {
        assert_json_snapshot!("default_rename_variables", new_rule());
    }

    #[test]
    fn serialize_no_globals_rule() {
        assert_json_snapshot!(
            "no_globals_rename_variables",
            Box::new(RenameVariables::new(empty())) as Box<dyn Rule>
        );
    }

    #[test]
    fn serialize_roblox_globals_rule() {
        let rule = Box::new(RenameVariables::new(
            globals::ROBLOX.to_vec().into_iter().map(String::from),
        ));

        assert_json_snapshot!("roblox_globals_rename_variables", rule as Box<dyn Rule>);
    }

    #[test]
    fn is_valid_identifier_is_true() {
        assert!(is_valid_identifier("hello"));
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("bar"));
    }

    #[test]
    fn is_valid_identifier_is_false() {
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("$hello"));
        assert!(!is_valid_identifier(" "));
        assert!(!is_valid_identifier("5"));
        assert!(!is_valid_identifier("1bar"));
    }
}
