use crate::nodes::{
    Block,
    DecimalNumber,
    Expression,
    LocalFunctionStatement,
    Prefix,
    StringExpression,
};
use crate::process::{NodeProcessorMut, NodeVisitorMut, ScopeMut, ScopeVisitorMut};
use crate::rules::{Rule, RuleConfigurationError, RuleProperties, RulePropertyValue};

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

    fn is_identifier_used(&self, identifier: &String) -> bool {
        self.identifiers.iter()
            .any(|set| set.contains(identifier))
    }

    fn insert_identifier(&mut self, identifier: &String) {
        if let Some(set) = self.identifiers.last_mut() {
            set.insert(identifier.clone());
        } else {
            let mut set = HashSet::new();
            set.insert(identifier.clone());
            self.identifiers.push(set);
        }
    }
}

impl ScopeMut for ValueInjection {
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
        self.insert_identifier(function.mutate_identifier());
    }
}

impl NodeProcessorMut for ValueInjection {
    fn process_expression(&mut self, expression: &mut Expression) {
        if self.is_identifier_used(&self.identifier) {
            return
        }

        let replace = match expression {
            Expression::Identifier(identifier) => &self.identifier == identifier,
            _ => false,
        };

        if replace {
            let new_expression = self.expression.clone();
            *expression = new_expression;
        }
    }

    fn process_prefix_expression(&mut self, prefix: &mut Prefix) {
        let replace = match prefix {
            Prefix::Identifier(identifier) => &self.identifier == identifier,
            _ => false,
        };

        if replace {
            let new_prefix = Prefix::Parenthese(self.expression.clone());
            *prefix = new_prefix;
        }
    }
}

pub const INJECT_GLOBAL_VALUE_RULE_NAME: &'static str = "inject_global_value";

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
            value: Expression::Nil,
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
            value: Expression::Nil,
        }
    }
}

impl Rule for InjectGlobalValue {
    fn process(&self, block: &mut Block) {
        let mut processor = ValueInjection::new(&self.identifier, self.value.clone());
        ScopeVisitorMut::visit_block(block, &mut processor);
    }

    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        if !properties.contains_key("identifier") {
            return Err(RuleConfigurationError::MissingProperty("identifier".to_owned()))
        }

        for (key, value) in properties {
            match key.as_str() {
                "identifier" => {
                    match value {
                        RulePropertyValue::String(identifier) => {
                            self.identifier = identifier;
                        }
                        _ => return Err(RuleConfigurationError::StringExpected(key)),
                    }
                }
                "value" => {
                    match value {
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
                    }
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
        rules.insert("identifier".to_owned(), RulePropertyValue::String(self.identifier.clone()));

        let property_value = match self.value {
            Expression::True => RulePropertyValue::Boolean(true),
            Expression::False => RulePropertyValue::Boolean(false),
            Expression::Nil => RulePropertyValue::None,
            _ => unreachable!("unexpected expression {:?}", self.value),
        };
        rules.insert("value".to_owned(), property_value);

        rules
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use insta::assert_json_snapshot;

    #[test]
    fn configure_without_identifier_property_should_error() {
        let result = json5::from_str::<Box<dyn Rule>>(r#"{
            rule: 'inject_global_value',
        }"#);

        match result {
            Ok(_) => panic!("should return an error"),
            Err(_) => {}
        }
    }

    #[test]
    fn deserialize_from_string_notation_should_error() {
        let result = json5::from_str::<Box<dyn Rule>>("'inject_global_value'");

        match result {
            Ok(_) => panic!("should return an error"),
            Err(_) => {}
        }
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
