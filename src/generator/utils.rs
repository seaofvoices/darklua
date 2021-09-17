//! A module that contains the main [LuaGenerator](trait.LuaGenerator.html) trait
//! and its implementations.

use crate::nodes;

use std::str::CharIndices;

pub fn is_relevant_for_spacing(character: &char) -> bool {
    character.is_ascii_alphabetic() || character.is_digit(10) || *character == '_'
}

pub fn break_long_string(last_str: &str) -> bool {
    if let Some(last_char) = last_str.chars().last() {
        last_char == '['
    } else {
        false
    }
}

pub fn break_variable_arguments(last_string: &str) -> bool {
    if let Some('.') = last_string.chars().last() {
        true
    } else if let Some(first_char) = last_string.chars().next() {
        first_char == '.' || first_char.is_digit(10)
    } else {
        false
    }
}

pub fn break_minus(last_string: &str) -> bool {
    if let Some(last_char) = last_string.chars().last() {
        last_char == '-'
    } else {
        false
    }
}

pub fn break_concat(last_string: &str) -> bool {
    if let Some('.') = last_string.chars().last() {
        true
    } else if let Some(first_char) = last_string.chars().next() {
        first_char == '.' || first_char.is_digit(10)
    } else {
        false
    }
}

pub fn find_not_escaped_from(pattern: char, chars: &mut CharIndices) -> Option<usize> {
    let mut escaped = false;
    chars.find_map(|(index, character)| {
        if escaped {
            escaped = false;
            None
        } else {
            match character {
                '\\' => {
                    escaped = true;
                    None
                }
                value => if value == pattern {
                    Some(index)
                } else {
                    None
                },
            }
        }
    })
}

pub fn ends_with_prefix(statement: &nodes::Statement) -> bool {
    use nodes::Statement::*;
    match statement {
        Assign(assign) => {
            if let Some(value) = assign.get_values().last() {
                expression_ends_with_call(value)
            } else {
                false
            }
        }
        CompoundAssign(assign) => {
            expression_ends_with_call(assign.get_value())
        }
        Call(_) => true,
        Repeat(repeat) => expression_ends_with_call(repeat.get_condition()),
        LocalAssign(assign) => {
            if let Some(value) = assign.get_values().last() {
                expression_ends_with_call(value)
            } else {
                false
            }
        }
        _ => false,
    }
}

pub fn starts_with_parenthese(statement: &nodes::Statement) -> bool {
    use nodes::{Statement, Variable};
    match statement {
        Statement::Assign(assign) => {
            if let Some(variable) = assign.get_variables().first() {
                match variable {
                    Variable::Identifier(_) => false,
                    Variable::Field(field) => field_starts_with_parenthese(field),
                    Variable::Index(index) => index_starts_with_parenthese(index),
                }
            } else {
                false
            }
        }
        Statement::CompoundAssign(assign) => {
            match assign.get_variable() {
                Variable::Identifier(_) => false,
                Variable::Field(field) => field_starts_with_parenthese(field),
                Variable::Index(index) => index_starts_with_parenthese(index),
            }
        }
        Statement::Call(call) => call_starts_with_parenthese(call),
        _ => false,
    }
}

fn expression_ends_with_call(expression: &nodes::Expression) -> bool {
    use nodes::Expression::*;

    match expression {
        Binary(binary) => expression_ends_with_call(binary.right()),
        Call(_)
        | Parenthese(_)
        | Identifier(_)
        | Field(_)
        | Index(_) => true,
        Unary(unary) => expression_ends_with_call(unary.get_expression()),
        _ => false,
    }
}

fn prefix_starts_with_parenthese(prefix: &nodes::Prefix) -> bool {
    use nodes::Prefix::*;
    match prefix {
        Parenthese(_) => true,
        Call(call) => call_starts_with_parenthese(call),
        Field(field) => field_starts_with_parenthese(field),
        Index(index) => index_starts_with_parenthese(index),
        Identifier(_) => false,
    }
}

#[inline]
fn call_starts_with_parenthese(call: &nodes::FunctionCall) -> bool {
    prefix_starts_with_parenthese(call.get_prefix())
}

#[inline]
fn field_starts_with_parenthese(field: &nodes::FieldExpression) -> bool {
    prefix_starts_with_parenthese(field.get_prefix())
}

#[inline]
fn index_starts_with_parenthese(index: &nodes::IndexExpression) -> bool {
    prefix_starts_with_parenthese(index.get_prefix())
}
