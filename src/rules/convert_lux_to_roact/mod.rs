use crate::nodes::{
    Arguments,
    Block,
    Expression,
    FieldExpression,
    FunctionCall,
    Prefix,
    LocalFunctionStatement,
    StringExpression,
    TableExpression,
    TableEntry,
    LUXAttribute,
    LUXAttributeName,
    LUXAttributeValue,
    LUXChild,
    LUXExpression,
    LUXElement,
    LUXElementName,
    LUXFragment,
};
use crate::process::{
    NodeProcessor,
    NodeVisitor,
    ScopeVisitor,
    Scope,
};
use crate::rules::{Rule, RuleConfigurationError, RuleProperties};

use std::collections::HashSet;

#[derive(Debug, Clone)]
struct RoactCodeGenerator {
    roact_identifier: String,
    identifiers: Vec<HashSet<String>>,
}

impl RoactCodeGenerator {
    fn is_identifier_used(&self, identifier: &str) -> bool {
        self.identifiers.iter()
            .any(|set| set.contains(identifier))
    }

    fn insert_identifier(&mut self, identifier: &str) {
        if let Some(set) = self.identifiers.last_mut() {
            set.insert(identifier.to_owned());
        } else {
            let mut set = HashSet::new();
            set.insert(identifier.to_owned());
            self.identifiers.push(set);
        }
    }

    fn roact_prefix(&self) -> Prefix {
        Prefix::from_name(&self.roact_identifier)
    }

    fn convert_lux_expression(&self, lux: &LUXExpression) -> Expression {
        match lux {
            LUXExpression::LUXElement(element) => self.convert_element(element).0,
            LUXExpression::LUXFragment(fragment) => self.convert_fragment(fragment),
        }
    }

    fn convert_element(&self, element: &LUXElement) -> (Expression, Option<Expression>) {
        let component_name = match element.get_name() {
            LUXElementName::Identifier(name) => {
                if self.is_identifier_used(name) {
                    Expression::Identifier(name.to_owned())
                } else {
                    StringExpression::from_value(name).into()
                }
            }
            LUXElementName::NamespacedName(_name) => {
                unimplemented!()
            }
            LUXElementName::Members(_root, _members) => {
                unimplemented!()
            }
        };

        let (props, key) = self.convert_attributes(element.get_attributes());

        let empty_props = match &props {
            Expression::Nil => true,
            Expression::Table(table) => table.is_empty(),
            _ => false,
        };

        let mut arguments = Arguments::default()
            .append_argument(component_name);

        if !empty_props {
            arguments.push_argument(props);
        }

        let expression = FunctionCall::new(
            FieldExpression::new(self.roact_prefix(), "createElement").into(),
            arguments,
            None,
        ).into();

        (expression, key)
    }

    fn convert_fragment(&self, fragment: &LUXFragment) -> Expression {
        FunctionCall::new(
            FieldExpression::new(self.roact_prefix(), "createFragment").into(),
            Arguments::default()
                .append_argument(self.convert_children(fragment.get_children())),
            None,
        ).into()
    }

    fn convert_children(&self, children: &Vec<LUXChild>) -> Expression {
        let mut current_children = Vec::new();
        let mut merge_lists = Vec::new();

        for child in children.iter() {
            match child {
                LUXChild::LUXElement(element) => {
                    let (expression, key) = self.convert_element(element);
                    let entry = if let Some(key) = key {
                        TableEntry::from_dictionary_entry(key, expression)
                    } else {
                        expression.into()
                    };
                    current_children.push(entry);
                }
                LUXChild::LUXFragment(fragment) => {
                    let expression = self.convert_fragment(fragment);
                    current_children.push(expression.into());
                }
                LUXChild::Expression(expression) => {
                    if let Some(expression) = expression {
                        current_children.push(expression.clone().into());
                    }
                }
                LUXChild::ExpandedExpression(expression) => {
                    let entries = current_children.drain(..)
                        .collect();
                    merge_lists.push(TableExpression::new(entries).into());
                    merge_lists.push(expression.clone());
                }
            }
        }

        if merge_lists.is_empty() {
            TableExpression::new(current_children).into()
        } else {
            unimplemented!()
        }
    }

    fn convert_attributes(&self, attributes: &Vec<LUXAttribute>) -> (Expression, Option<Expression>) {
        let mut current_props = Vec::new();
        let mut merge_lists = Vec::new();
        let mut key = None;

        for attribute in attributes.iter() {
            match attribute {
                LUXAttribute::Named(attribute) => {
                    let mut value = Some(if let Some(attribute_value) = attribute.get_value() {
                        match attribute_value {
                            LUXAttributeValue::DoubleQuoteString(string)
                            | LUXAttributeValue::SingleQuoteString(string) => {
                                StringExpression::from_value(string).into()
                            }
                            LUXAttributeValue::LuaExpression(expression) => {
                                expression.clone()
                            }
                            LUXAttributeValue::LUXElement(element) => {
                                self.convert_element(element).0
                            }
                            LUXAttributeValue::LUXFragment(fragment) => {
                                self.convert_fragment(fragment)
                            }
                        }
                    } else {
                        Expression::True
                    });

                    let prop: Option<Expression> = match attribute.get_name() {
                        LUXAttributeName::Identifier(identifier) => {
                            Some(StringExpression::from_value(identifier).into())
                        }
                        LUXAttributeName::NamespacedName(namespaced_name) => {
                            match namespaced_name.get_namespace().as_ref() {
                                "event" => {
                                    Some(FieldExpression::new(
                                        FieldExpression::new(
                                            self.roact_prefix(),
                                            "Event",
                                        ).into(),
                                        namespaced_name.get_member()
                                    ).into())
                                }
                                "changed" => {
                                    Some(FieldExpression::new(
                                        FieldExpression::new(
                                            self.roact_prefix(),
                                            "Changed",
                                        ).into(),
                                        namespaced_name.get_member()
                                    ).into())
                                }
                                "roact" => {
                                    match namespaced_name.get_member().as_ref() {
                                        "key" => {
                                            key = value.take();
                                            None
                                        }
                                        "ref" => {
                                            Some(FieldExpression::new(
                                                self.roact_prefix(),
                                                "Ref",
                                            ).into())
                                        }
                                        _ => {
                                            unimplemented!()
                                        }
                                    }
                                }
                                _ => {
                                    unimplemented!()
                                }
                            }
                        }
                    };

                    if let Some((prop, value)) = prop.zip(value) {
                        current_props.push(TableEntry::from_dictionary_entry(
                            prop,
                            value,
                        ));
                    }
                }
                LUXAttribute::Spread(expression) => {
                    let entries: Vec<_> = current_props.drain(..)
                        .collect();
                    if !entries.is_empty() {
                        merge_lists.push(TableExpression::new(entries).into());
                    }
                    merge_lists.push(expression.clone());
                }
            }
        }

        if merge_lists.is_empty() {
            (TableExpression::new(current_props).into(), key)
        } else {
            if current_props.is_empty() {
                if merge_lists.len() == 1 {
                    (merge_lists.pop().unwrap(), key)
                } else {
                    unimplemented!()
                }
            } else {
                unimplemented!()
            }
        }
    }
}

impl Default for RoactCodeGenerator {
    fn default() -> Self {
        Self {
            roact_identifier: "Roact".to_owned(),
            identifiers: Vec::new(),
        }
    }
}

impl Scope for RoactCodeGenerator {
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
        self.insert_identifier(function.get_name());
    }
}

impl NodeProcessor for RoactCodeGenerator {
    fn process_expression(&mut self, expression: &mut Expression) {
        match expression {
            Expression::LUX(lux_expression) => {
                let new_expression = self.convert_lux_expression(lux_expression);
                *expression = new_expression;
            }
            _ => {}
        }
    }
}

pub const CONVERT_LUX_TO_ROACT_CODE_RULE_NAME: &'static str = "convert_lux_to_roact_code";

/// A rule that removes LUX expressions and generates code for Roact.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConvertLUXToRoactCode {}

impl Rule for ConvertLUXToRoactCode {
    fn process(&self, block: &mut Block) {
        let mut processor = RoactCodeGenerator::default();
        ScopeVisitor::visit_block(block, &mut processor);
    }

    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        // TODO: make the roact identifier configurable
        for (key, _value) in properties {
            return Err(RuleConfigurationError::UnexpectedProperty(key))
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        CONVERT_LUX_TO_ROACT_CODE_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     use insta::assert_json_snapshot;

//     fn new_rule() -> ConvertLUXToRoactCode {
//         ConvertLUXToRoactCode::default()
//     }

//     #[test]
//     fn serialize_default_rule() {
//         let rule: Box<dyn Rule> = Box::new(new_rule());

//         assert_json_snapshot!("default_convert_lux_to_roact_code", rule);
//     }
// }
