use crate::nodes::{Block, LocalAssignStatement, LocalFunctionStatement};
use crate::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleMetadata, RuleProperties,
};

use serde::ser::{Serialize, Serializer};

use super::verify_no_rule_properties;

#[derive(Debug, Default)]
struct ConvertConstToLocalProcessor;

impl NodeProcessor for ConvertConstToLocalProcessor {
    fn process_local_assign_statement(&mut self, assign: &mut LocalAssignStatement) {
        if assign.is_const() {
            assign.set_local();
        }
    }

    fn process_local_function_statement(&mut self, function: &mut LocalFunctionStatement) {
        if function.is_const() {
            function.set_local();
        }
    }
}

pub const CONVERT_CONST_TO_LOCAL_RULE_NAME: &str = "convert_const_to_local";

/// A rule that converts Luau `const` declarations into `local` declarations.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConvertConstToLocal {
    metadata: RuleMetadata,
}

impl FlawlessRule for ConvertConstToLocal {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let mut processor = ConvertConstToLocalProcessor;
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for ConvertConstToLocal {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        CONVERT_CONST_TO_LOCAL_RULE_NAME
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

impl Serialize for ConvertConstToLocal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(CONVERT_CONST_TO_LOCAL_RULE_NAME)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::rules::Rule;

    use insta::assert_json_snapshot;

    fn new_rule() -> ConvertConstToLocal {
        ConvertConstToLocal::default()
    }

    #[test]
    fn serialize_default_rule() {
        assert_json_snapshot!(new_rule(), @r###""convert_const_to_local""###);
    }

    #[test]
    fn configure_with_extra_field_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
            rule: 'convert_const_to_local',
            prop: "something",
        }"#,
        );
        insta::assert_snapshot!(result.unwrap_err().to_string(), @"unexpected field 'prop' at line 1 column 1")
    }
}
