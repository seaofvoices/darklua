use crate::nodes::{
    Block, DecimalNumber, Expression, LocalFunctionStatement, ParentheseExpression, Prefix,
    StringExpression,
};
use crate::process::{NodeProcessor, NodeVisitor, Scope, ScopeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
    RulePropertyValue,
};

use std::collections::HashSet;

#[derive(Debug, Clone)]
struct ValueInjection {
    identifier: String,
    expression: Expression,
    identifiers: Vec<HashSet<String>>,
}

impl ValueInjection {
    pub fn new<S: Into<String>, E: Into<Expression>>(identifier: S, expression: E) -> Self {
        Self {
            identifier: identifier.into(),
            expression: expression.into(),
            identifiers: Vec::new(),
        }
    }

    fn is_identifier_used(&self, identifier: &str) -> bool {
        self.identifiers.iter().any(|set| set.contains(identifier))
    }

    fn insert_identifier(&mut self, identifier: &str) {
        if let Some(set) = self.identifiers.last_mut() {
            set.insert(identifier.to_string());
        } else {
            let mut set = HashSet::new();
            set.insert(identifier.to_string());
            self.identifiers.push(set);
        }
    }
}

impl Scope for ValueInjection {
    fn push(&mut self) {
        self.identifiers.push(HashSet::new())
    }

    fn pop(&mut self) {
        self.identifiers.pop();
    }

    fn insert(&mut self, identifier: &mut String) {
        self.insert_identifier(identifier);
    }

    fn insert_local(&mut self, identifier: &mut String, _value: Option<&mut Expression>) {
        self.insert_identifier(identifier);
    }

    fn insert_local_function(&mut self, function: &mut LocalFunctionStatement) {
        self.insert_identifier(function.mutate_identifier().get_name());
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

    pub fn string<S: Into<String>>(identifier: S, value: S) -> Self {
        Self {
            identifier: identifier.into(),
            value: StringExpression::from_value(value).into(),
        }
    }

    pub fn float<S: Into<String>>(identifier: S, value: f64) -> Self {
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
    fn flawless_process(&self, block: &mut Block, _: &mut Context) {
        let mut processor = ValueInjection::new(&self.identifier, self.value.clone());
        ScopeVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for InjectGlobalValue {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        if !properties.contains_key("identifier") {
            return Err(RuleConfigurationError::MissingProperty(
                "identifier".to_owned(),
            ));
        }

        for (key, value) in properties {
            match key.as_str() {
                "identifier" => match value {
                    RulePropertyValue::String(identifier) => {
                        self.identifier = identifier;
                    }
                    _ => return Err(RuleConfigurationError::StringExpected(key)),
                },
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

        let property_value = match self.value {
            Expression::True(_) => RulePropertyValue::Boolean(true),
            Expression::False(_) => RulePropertyValue::Boolean(false),
            Expression::Nil(_) => RulePropertyValue::None,
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
}
