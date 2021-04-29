use crate::nodes::{
    Arguments,
    AssignStatement,
    Block,
    Expression,
    FieldExpression,
    FunctionCall,
    LastStatement,
    Prefix,
    GenericForStatement,
    IndexExpression,
    LocalAssignStatement,
    LocalFunctionStatement,
    NumericForStatement,
    Statement,
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
    Scope,
};
use crate::rules::convert_lux_to_roact::IdentifierCollector;

use std::collections::HashSet;

#[derive(Debug)]
pub struct RoactCodeGenerator {
    roact_identifier: String,
    identifiers: Vec<HashSet<String>>,
    identifier_generator: IdentifierCollector,
    merge_identifier: Option<String>,
}

const MERGE_FUNCTION_IDENTIFIER: &'static str = "_DARKLUA_SHALLOW_MERGE";

fn create_merge_function<I: Into<String>>(identifier: I) -> Statement {
    let new = "new";
    let index = "index";
    let key = "key";
    let value = "value";
    LocalFunctionStatement::from_name(
        identifier,
        Block::new(
            vec![
                LocalAssignStatement::from_variable(new)
                    .with_value(TableExpression::default())
                    .into(),
                NumericForStatement::new(
                    index.to_owned(),
                    Expression::from(1_f64),
                    FunctionCall::from_name("select")
                        .with_arguments(Arguments::Tuple(vec![
                            StringExpression::from_value("#").into(),
                            Expression::VariableArguments,
                        ]))
                        .into(),
                    None,
                    Block::new(
                        vec![
                            GenericForStatement::new(
                                vec![key.to_owned(), value.to_owned()],
                                vec![
                                    FunctionCall::from_name("pairs")
                                        .with_arguments(Arguments::Tuple(vec![
                                            FunctionCall::from_name("select")
                                                .with_arguments(Arguments::Tuple(vec![
                                                    Expression::Identifier(index.to_owned()),
                                                    Expression::VariableArguments,
                                                ]))
                                                .into(),
                                        ]))
                                        .into(),
                                ],
                                Block::new(
                                    vec![
                                        AssignStatement::new(
                                            vec![
                                                IndexExpression::new(
                                                    Prefix::from_name(new),
                                                    Expression::Identifier(key.to_owned())
                                                ).into()
                                            ],
                                            vec![
                                                Expression::Identifier(value.to_owned())
                                            ]
                                        ).into(),
                                    ],
                                    None,
                                )
                            ).into(),
                        ],
                        None,
                    ),
                ).into(),
            ],
            Some(LastStatement::Return(
                vec![Expression::Identifier(new.to_owned())]
            ))
        )
    )
    .variadic()
    .into()
}

impl RoactCodeGenerator {
    pub fn new(identifier_generator: IdentifierCollector) -> Self {
        Self {
            roact_identifier: "Roact".to_owned(),
            identifiers: Vec::new(),
            identifier_generator,
            merge_identifier: None,
        }
    }

    pub fn post_process_block(&self, block: &mut Block) {
        if let Some(identifier) = &self.merge_identifier {
            let statements = block.mutate_statements();
            let merge_definition = create_merge_function(identifier);
            statements.insert(0, merge_definition.into());
        }
    }

    fn get_merge_function(&mut self, arguments: Vec<Expression>) -> Expression {
        let merge_function_name = &self.merge_identifier.clone()
            .unwrap_or_else(|| {
                let identifier = self.identifier_generator
                    .try_get_identifier(MERGE_FUNCTION_IDENTIFIER);
                self.merge_identifier = Some(identifier.clone());
                identifier
            });

        FunctionCall::from_name(merge_function_name)
            .with_arguments(Arguments::Tuple(arguments))
            .into()
    }

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

    fn convert_lux_expression(&mut self, lux: &LUXExpression) -> Expression {
        match lux {
            LUXExpression::LUXElement(element) => self.convert_element(element).0,
            LUXExpression::LUXFragment(fragment) => self.convert_fragment(fragment),
        }
    }

    fn convert_element(&mut self, element: &LUXElement) -> (Expression, Option<Expression>) {
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

    fn convert_fragment(&mut self, fragment: &LUXFragment) -> Expression {
        FunctionCall::new(
            FieldExpression::new(self.roact_prefix(), "createFragment").into(),
            Arguments::default()
                .append_argument(self.convert_children(fragment.get_children())),
            None,
        ).into()
    }

    fn convert_children(&mut self, children: &Vec<LUXChild>) -> Expression {
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

    fn convert_attributes(&mut self, attributes: &Vec<LUXAttribute>) -> (Expression, Option<Expression>) {
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
            if !current_props.is_empty() {
                merge_lists.push(TableExpression::new(current_props).into());
            }

            if merge_lists.len() == 1 {
                (merge_lists.pop().unwrap(), key)
            } else {
                let expression = self.get_merge_function(merge_lists);
                (expression, key)
            }
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
