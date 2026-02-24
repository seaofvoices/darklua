use regex::Regex;

use crate::nodes::*;
use crate::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleMetadata, RuleProperties,
};

pub const REMOVE_ATTRIBUTE_RULE_NAME: &str = "remove_attribute";

#[derive(Debug, Default)]
struct RemoveAttributeProcessor;

impl NodeProcessor for RemoveAttributeProcessor {
    fn process_function_statement(&mut self, function: &mut FunctionStatement) {
        function.mutate_attributes().clear_attributes();
    }

    fn process_local_function_statement(&mut self, function: &mut LocalFunctionStatement) {
        function.mutate_attributes().clear_attributes();
    }

    fn process_function_expression(&mut self, function: &mut FunctionExpression) {
        function.mutate_attributes().clear_attributes();
    }
}

struct FilterAttributeProcessor<'a> {
    match_patterns: &'a Vec<Regex>,
}

impl<'a> FilterAttributeProcessor<'a> {
    pub fn new(match_patterns: &'a Vec<Regex>) -> Self {
        Self { match_patterns }
    }

    fn should_remove_name(&self, attribute_name: &str) -> bool {
        self.match_patterns
            .iter()
            .any(|pattern| pattern.is_match(attribute_name))
    }

    fn should_remove(&self, attribute: &mut Attribute) -> bool {
        match attribute {
            Attribute::Name(named) => self.should_remove_name(named.get_identifier().get_name()),
            Attribute::Group(group) => {
                group.filter_attributes(|element| {
                    self.should_remove_name(element.name().get_name())
                });
                !group.is_empty()
            }
        }
    }
}

impl<'a> NodeProcessor for FilterAttributeProcessor<'a> {
    fn process_function_statement(&mut self, function: &mut FunctionStatement) {
        function
            .mutate_attributes()
            .filter_mut_attributes(|attribute| !self.should_remove(attribute));
    }

    fn process_local_function_statement(&mut self, function: &mut LocalFunctionStatement) {
        function
            .mutate_attributes()
            .filter_mut_attributes(|attribute| !self.should_remove(attribute));
    }

    fn process_function_expression(&mut self, function: &mut FunctionExpression) {
        function
            .mutate_attributes()
            .filter_mut_attributes(|attribute| !self.should_remove(attribute));
    }
}

/// A rule that removes function attributes.
///
/// When configured with a `match` parameter containing regex patterns,
/// only attributes whose names match the patterns are removed.
/// When `match` is empty (default), all attributes are removed.
#[derive(Debug, Default)]
pub struct RemoveAttribute {
    metadata: RuleMetadata,
    r#match: Vec<Regex>,
}

impl RemoveAttribute {
    /// Adds a regex pattern to match against attribute names.
    pub fn with_match(mut self, match_pattern: &str) -> Self {
        match Regex::new(match_pattern) {
            Ok(regex) => {
                self.r#match.push(regex);
            }
            Err(err) => {
                log::warn!(
                    "unable to compile regex pattern `{}`: {}",
                    match_pattern,
                    err
                );
            }
        }
        self
    }
}

impl FlawlessRule for RemoveAttribute {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        if self.r#match.is_empty() {
            let mut processor = RemoveAttributeProcessor;
            DefaultVisitor::visit_block(block, &mut processor);
        } else {
            let mut processor = FilterAttributeProcessor::new(&self.r#match);
            DefaultVisitor::visit_block(block, &mut processor);
        }
    }
}

impl RuleConfiguration for RemoveAttribute {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        for (key, value) in properties {
            match key.as_str() {
                "match" => {
                    self.r#match = value.expect_regex_list(&key)?;
                }
                _ => return Err(RuleConfigurationError::UnexpectedProperty(key)),
            }
        }
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_ATTRIBUTE_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }

    fn set_metadata(&mut self, metadata: RuleMetadata) {
        self.metadata = metadata;
    }

    fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rule;

    fn new_rule() -> RemoveAttribute {
        RemoveAttribute::default()
    }

    #[test]
    fn test_serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        insta::assert_json_snapshot!(rule, @r#""remove_attribute""#);
    }

    #[test]
    fn test_configure_with_extra_field_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
                rule: 'remove_attribute',
                unexpected: true,
            }"#,
        );

        insta::assert_snapshot!(result.unwrap_err(), @"unexpected field 'unexpected' at line 1 column 1");
    }

    #[test]
    fn test_configure_with_invalid_regex_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
                rule: 'remove_attribute',
                match: ['[invalid'],
            }"#,
        );

        insta::assert_snapshot!(result.unwrap_err(), @r###"
        unexpected value for field 'match': invalid regex provided `[invalid`
          regex parse error:
            [invalid
            ^
        error: unclosed character class at line 1 column 1
        "###);
    }
}
