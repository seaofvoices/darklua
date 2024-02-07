use crate::nodes::{
    Block, DecimalNumber, Expression, ParentheseExpression, Prefix, StringExpression, UnaryOperator,
};
use crate::process::{IdentifierTracker, NodeProcessor, NodeVisitor, ScopeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
    RulePropertyValue,
};

use std::{env, ops};

use super::{verify_property_collisions, verify_required_properties};

#[derive(Debug, Clone)]
struct ValueInjection {
    identifier: String,
    expression: Expression,
    identifier_tracker: IdentifierTracker,
}

impl ValueInjection {
    pub fn new<S: Into<String>, E: Into<Expression>>(identifier: S, expression: E) -> Self {
        Self {
            identifier: identifier.into(),
            expression: expression.into(),
            identifier_tracker: IdentifierTracker::default(),
        }
    }
}

impl ops::Deref for ValueInjection {
    type Target = IdentifierTracker;

    fn deref(&self) -> &Self::Target {
        &self.identifier_tracker
    }
}

impl ops::DerefMut for ValueInjection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.identifier_tracker
    }
}

impl NodeProcessor for ValueInjection {
    fn process_expression(&mut self, expression: &mut Expression) {
        let replace = match expression {
            Expression::Identifier(identifier) => {
                &self.identifier == identifier.get_name()
                    && !self.is_identifier_used(&self.identifier)
            }
            Expression::Field(field) => {
                &self.identifier == field.get_field().get_name()
                    && !self.is_identifier_used("_G")
                    && matches!(field.get_prefix(), Prefix::Identifier(prefix) if prefix.get_name() == "_G")
            }
            Expression::Index(index) => {
                !self.is_identifier_used("_G")
                    && matches!(index.get_index(), Expression::String(string) if string.get_value() == self.identifier)
                    && matches!(index.get_prefix(), Prefix::Identifier(prefix) if prefix.get_name() == "_G")
            }
            _ => false,
        };

        if replace {
            let new_expression = self.expression.clone();
            *expression = new_expression;
        }
    }

    fn process_prefix_expression(&mut self, prefix: &mut Prefix) {
        let replace = match prefix {
            Prefix::Identifier(identifier) => &self.identifier == identifier.get_name(),
            _ => false,
        };

        if replace {
            let new_prefix = ParentheseExpression::new(self.expression.clone()).into();
            *prefix = new_prefix;
        }
    }
}

pub const INJECT_GLOBAL_VALUE_RULE_NAME: &str = "inject_global_value";

/// A rule to replace global variables with values.
#[derive(Debug, PartialEq, Eq)]
pub struct InjectGlobalValue {
    identifier: String,
    value: Expression,
}

impl InjectGlobalValue {
    pub fn nil<S: Into<String>>(identifier: S) -> Self {
        Self {
            identifier: identifier.into(),
            value: Expression::nil(),
        }
    }

    pub fn boolean<S: Into<String>>(identifier: S, value: bool) -> Self {
        Self {
            identifier: identifier.into(),
            value: Expression::from(value),
        }
    }

    pub fn string<S: Into<String>, S2: Into<String>>(identifier: S, value: S2) -> Self {
        Self {
            identifier: identifier.into(),
            value: StringExpression::from_value(value).into(),
        }
    }

    pub fn number<S: Into<String>>(identifier: S, value: f64) -> Self {
        Self {
            identifier: identifier.into(),
            value: Expression::from(value),
        }
    }
}

impl Default for InjectGlobalValue {
    fn default() -> Self {
        Self {
            identifier: "".to_owned(),
            value: Expression::nil(),
        }
    }
}

impl FlawlessRule for InjectGlobalValue {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let mut processor = ValueInjection::new(&self.identifier, self.value.clone());
        ScopeVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for InjectGlobalValue {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_required_properties(&properties, &["identifier"])?;
        verify_property_collisions(&properties, &["value", "env"])?;

        for (key, value) in properties {
            match key.as_str() {
                "identifier" => {
                    self.identifier = value.expect_string(&key)?;
                }
                "value" => match value {
                    RulePropertyValue::None => {}
                    RulePropertyValue::String(value) => {
                        self.value = StringExpression::from_value(value).into();
                    }
                    RulePropertyValue::Boolean(value) => {
                        self.value = Expression::from(value);
                    }
                    RulePropertyValue::Usize(value) => {
                        self.value = DecimalNumber::new(value as f64).into();
                    }
                    RulePropertyValue::Float(value) => {
                        self.value = Expression::from(value);
                    }
                    _ => return Err(RuleConfigurationError::UnexpectedValueType(key)),
                },
                "env" => {
                    let variable_name = value.expect_string(&key)?;
                    if let Some(os_value) = env::var_os(&variable_name) {
                        if let Some(value) = os_value.to_str() {
                            self.value = StringExpression::from_value(value).into();
                        } else {
                            return Err(RuleConfigurationError::UnexpectedValue {
                                property: key,
                                message: format!(
                                    "invalid string assigned to the `{}` environment variable",
                                    &variable_name,
                                ),
                            });
                        }
                    } else {
                        log::warn!(
                            "environment variable `{}` is not defined. The rule `{}` will use `nil`",
                            variable_name,
                            INJECT_GLOBAL_VALUE_RULE_NAME,
                        );
                    };
                }
                _ => return Err(RuleConfigurationError::UnexpectedProperty(key)),
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        INJECT_GLOBAL_VALUE_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        let mut rules = RuleProperties::new();
        rules.insert(
            "identifier".to_owned(),
            RulePropertyValue::String(self.identifier.clone()),
        );

        let property_value = match &self.value {
            Expression::True(_) => RulePropertyValue::Boolean(true),
            Expression::False(_) => RulePropertyValue::Boolean(false),
            Expression::Nil(_) => RulePropertyValue::None,
            Expression::Number(number) => {
                let value = number.compute_value();
                if value.trunc() == value && value >= 0.0 && value < usize::MAX as f64 {
                    RulePropertyValue::Usize(value as usize)
                } else {
                    RulePropertyValue::Float(value)
                }
            }
            Expression::String(string) => RulePropertyValue::from(string.get_value()),
            Expression::Unary(unary) => {
                if matches!(unary.operator(), UnaryOperator::Minus) {
                    if let Expression::Number(number) = unary.get_expression() {
                        RulePropertyValue::Float(-number.compute_value())
                    } else {
                        unreachable!(
                            "unexpected expression for unary minus {:?}",
                            unary.get_expression()
                        );
                    }
                } else {
                    unreachable!("unexpected unary operator {:?}", unary.operator());
                }
            }
            _ => unreachable!("unexpected expression {:?}", self.value),
        };
        rules.insert("value".to_owned(), property_value);

        rules
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rules::Rule;

    use insta::assert_json_snapshot;

    #[test]
    fn configure_without_identifier_property_should_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
            rule: 'inject_global_value',
        }"#,
        );

        assert!(result.is_err());
    }

    #[test]
    fn configure_with_value_and_env_properties_should_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
            rule: 'inject_global_value',
            value: false,
            env: "VAR",
        }"#,
        );

        assert!(result.is_err());
    }

    #[test]
    fn deserialize_from_string_notation_should_error() {
        let result = json5::from_str::<Box<dyn Rule>>("'inject_global_value'");

        assert!(result.is_err());
    }

    #[test]
    fn serialize_inject_nil_as_foo() {
        let rule: Box<dyn Rule> = Box::new(InjectGlobalValue::nil("foo"));

        assert_json_snapshot!("inject_nil_value_as_foo", rule);
    }

    #[test]
    fn serialize_inject_true_as_foo() {
        let rule: Box<dyn Rule> = Box::new(InjectGlobalValue::boolean("foo", true));

        assert_json_snapshot!("inject_true_value_as_foo", rule);
    }

    #[test]
    fn serialize_inject_false_as_foo() {
        let rule: Box<dyn Rule> = Box::new(InjectGlobalValue::boolean("foo", false));

        assert_json_snapshot!("inject_false_value_as_foo", rule);
    }

    #[test]
    fn serialize_inject_string_as_var() {
        let rule: Box<dyn Rule> = Box::new(InjectGlobalValue::string("VAR", "hello"));

        assert_json_snapshot!("inject_hello_value_as_var", rule);
    }

    #[test]
    fn serialize_inject_integer_as_var() {
        let rule: Box<dyn Rule> = Box::new(InjectGlobalValue::number("VAR", 1.0));

        assert_json_snapshot!("inject_integer_value_as_var", rule);
    }

    #[test]
    fn serialize_inject_negative_integer_as_var() {
        let rule: Box<dyn Rule> = Box::new(InjectGlobalValue::number("VAR", -100.0));

        assert_json_snapshot!("inject_negative_integer_value_as_var", rule);
    }

    #[test]
    fn serialize_inject_float_as_var() {
        let rule: Box<dyn Rule> = Box::new(InjectGlobalValue::number("VAR", 123.45));

        assert_json_snapshot!("inject_float_value_as_var", rule);
    }
}
