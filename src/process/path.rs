use std::str::FromStr;

use crate::nodes::{
    AnyExpressionRef, AnyNodeRef, AnyStatementRef, Arguments, Block, Expression, FieldExpression,
    FunctionCall, IndexExpression, LastStatement, ParentheseExpression, Prefix, Statement,
    TableExpression,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Component {
    Statement(usize),
    Expression(usize),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct NodePath {
    components: Vec<Component>,
}

impl NodePath {
    #[inline]
    fn push_statement(&mut self, index: usize) {
        self.components.push(Component::Statement(index));
    }

    #[inline]
    fn push_expression(&mut self, index: usize) {
        self.components.push(Component::Expression(index));
    }

    fn with_statement(mut self, index: usize) -> Self {
        self.push_statement(index);
        self
    }

    fn with_expression(mut self, index: usize) -> Self {
        self.push_expression(index);
        self
    }

    fn resolve<'a>(&self, block: &'a Block) -> Option<AnyNodeRef<'a>> {
        let mut components = self.components.iter();

        let mut current = match components.next()? {
            Component::Statement(index) => AnyNodeRef::from(block.get_statement(*index)?),
            Component::Expression(_) => return None,
        };

        while let Some(current_component) = components.next() {
            current = match current_component {
                Component::Statement(index) => {
                    let next_statement = match current {
                        AnyNodeRef::AnyStatement(statement) => match statement {
                            AnyStatementRef::Statement(statement) => match statement {
                                Statement::Do(do_statement) => {
                                    do_statement.get_block().get_statement(*index)?
                                }
                                Statement::Function(function) => {
                                    function.get_block().get_statement(*index)?
                                }
                                Statement::GenericFor(generic_for) => {
                                    generic_for.get_block().get_statement(*index)?
                                }
                                Statement::If(if_statement) => {
                                    if_statement.get_statement(*index)?
                                }
                                Statement::LocalFunction(function) => {
                                    function.get_block().get_statement(*index)?
                                }
                                Statement::NumericFor(numeric_for) => {
                                    numeric_for.get_block().get_statement(*index)?
                                }
                                Statement::Repeat(repeat) => {
                                    repeat.get_block().get_statement(*index)?
                                }
                                Statement::While(while_statement) => {
                                    while_statement.get_block().get_statement(*index)?
                                }
                                Statement::Assign(_)
                                | Statement::Call(_)
                                | Statement::CompoundAssign(_)
                                | Statement::LocalAssign(_) => return None,
                            },
                            AnyStatementRef::LastStatement(_) => return None,
                        },
                        AnyNodeRef::AnyExpression(any_expression) => match any_expression {
                            AnyExpressionRef::Expression(expression) => match expression {
                                Expression::Function(function) => {
                                    function.get_block().get_statement(*index)?
                                }
                                Expression::Binary(_)
                                | Expression::Call(_)
                                | Expression::False(_)
                                | Expression::Field(_)
                                | Expression::Identifier(_)
                                | Expression::If(_)
                                | Expression::Index(_)
                                | Expression::Nil(_)
                                | Expression::Number(_)
                                | Expression::Parenthese(_)
                                | Expression::String(_)
                                | Expression::Table(_)
                                | Expression::True(_)
                                | Expression::Unary(_)
                                | Expression::VariableArguments(_) => return None,
                            },
                            AnyExpressionRef::Prefix(prefix) => match prefix {
                                Prefix::Call(_)
                                | Prefix::Field(_)
                                | Prefix::Identifier(_)
                                | Prefix::Index(_)
                                | Prefix::Parenthese(_) => return None,
                            },
                            AnyExpressionRef::Arguments(arguments) => match arguments {
                                Arguments::Tuple(_)
                                | Arguments::String(_)
                                | Arguments::Table(_) => todo!(),
                            },
                        },
                    };
                    AnyNodeRef::from(next_statement)
                }
                Component::Expression(index) => {
                    let next_expression = match current {
                        AnyNodeRef::AnyStatement(any_statement) => match any_statement {
                            AnyStatementRef::Statement(statement) => match statement {
                                Statement::Assign(_) => todo!(),
                                Statement::Do(_) => todo!(),
                                Statement::Call(call) => {
                                    resolve_call_expression(*index, call)?.into()
                                }
                                Statement::CompoundAssign(_) => todo!(),
                                Statement::Function(_) => todo!(),
                                Statement::GenericFor(_) => todo!(),
                                Statement::If(_) => todo!(),
                                Statement::LocalAssign(assign) => {
                                    assign.iter_values().skip(*index).next()?.into()
                                }
                                Statement::LocalFunction(_) => todo!(),
                                Statement::NumericFor(_) => todo!(),
                                Statement::Repeat(repeat) => match index {
                                    0 => repeat.get_condition().into(),
                                    _ => return None,
                                },
                                Statement::While(while_statement) => match index {
                                    0 => while_statement.get_condition().into(),
                                    _ => return None,
                                },
                            },
                            AnyStatementRef::LastStatement(last_statement) => {
                                match last_statement {
                                    LastStatement::Break(_) => return None,
                                    LastStatement::Continue(_) => return None,
                                    LastStatement::Return(return_statement) => {
                                        AnyExpressionRef::from(
                                            return_statement
                                                .iter_expressions()
                                                .skip(*index)
                                                .next()?,
                                        )
                                    }
                                }
                            }
                        },
                        AnyNodeRef::AnyExpression(any_expression) => match any_expression {
                            AnyExpressionRef::Expression(expression) => match expression {
                                Expression::Binary(binary) => match index {
                                    0 => binary.left().into(),
                                    1 => binary.right().into(),
                                    _ => return None,
                                },
                                Expression::Call(call) => resolve_call_expression(*index, call)?,
                                Expression::Field(field) => {
                                    resolve_field_expression(*index, field)?
                                }
                                Expression::If(if_statement) => {
                                    if_statement.get_expression(*index)?.into()
                                }
                                Expression::Index(index_expression) => {
                                    resolve_index_expression(*index, index_expression)?
                                }
                                Expression::Parenthese(parentheses) => {
                                    resolve_parentheses_expression(*index, parentheses)?
                                }
                                Expression::Table(table) => {
                                    resolve_table_expression(*index, table)?
                                }
                                Expression::Unary(unary) => match index {
                                    0 => unary.get_expression().into(),
                                    _ => return None,
                                },
                                Expression::False(_)
                                | Expression::Function(_)
                                | Expression::Identifier(_)
                                | Expression::Nil(_)
                                | Expression::Number(_)
                                | Expression::String(_)
                                | Expression::True(_)
                                | Expression::VariableArguments(_) => return None,
                            },
                            AnyExpressionRef::Prefix(prefix) => match prefix {
                                Prefix::Call(call) => resolve_call_expression(*index, call)?,
                                Prefix::Field(field) => resolve_field_expression(*index, field)?,
                                Prefix::Index(index_expression) => {
                                    resolve_index_expression(*index, index_expression)?
                                }
                                Prefix::Parenthese(parentheses) => {
                                    resolve_parentheses_expression(*index, parentheses)?
                                }
                                Prefix::Identifier(_) => return None,
                            },
                            AnyExpressionRef::Arguments(arguments) => match arguments {
                                Arguments::Tuple(tuple) => {
                                    tuple.iter_values().skip(*index).next()?.into()
                                }
                                Arguments::String(_) => todo!(),
                                Arguments::Table(table) => resolve_table_expression(*index, table)?,
                            },
                        },
                    };

                    AnyNodeRef::from(next_expression)
                }
            }
        }

        Some(current)
    }
}

const STATEMENT_DELIMITER: &str = "/";
const STATEMENT_DELIMITER_CHAR: char = '/';
const EXPRESSION_DELIMITER: &str = ":";
const EXPRESSION_DELIMITER_CHAR: char = ':';

impl FromStr for NodePath {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut path = Self::default();

        if !string.is_ascii() {
            return Err(format_node_path_parse_error(
                string,
                "contains non-ascii characters",
            ));
        }

        for component in string
            .split_inclusive(|c| matches!(c, STATEMENT_DELIMITER_CHAR | EXPRESSION_DELIMITER_CHAR))
        {
            if component.len() == 0 {
                return Err(format_node_path_parse_error(string, ""));
            }

            let (index_string, delimiter) = component.split_at(component.len().saturating_sub(1));

            let index = usize::from_str_radix(index_string, 10)
                .map_err(|err| format_node_path_parse_error(string, &err.to_string()))?;

            match delimiter {
                STATEMENT_DELIMITER => path.push_statement(index),
                EXPRESSION_DELIMITER => path.push_expression(index),
                "" => {
                    return Err(format_node_path_parse_error(
                        string,
                        &format!("missing delimiter"),
                    ))
                }
                _ => {
                    return Err(format_node_path_parse_error(
                        string,
                        &format!("unexpected delimiter `{}`", delimiter),
                    ))
                }
            }
        }

        Ok(path)
    }
}

fn format_node_path_parse_error(input: &str, reason: &str) -> String {
    if reason.len() == 0 {
        format!("unable to parse path `{}`", input)
    } else {
        format!("unable to parse path `{}`: {}", input, reason)
    }
}

impl ToString for NodePath {
    fn to_string(&self) -> String {
        self.components
            .iter()
            .map(|component| match component {
                Component::Statement(index) => format!("{}/", index),
                Component::Expression(index) => format!("{}:", index),
            })
            .collect()
    }
}

#[inline]
fn resolve_call_expression<'a>(
    index: usize,
    call: &'a FunctionCall,
) -> Option<AnyExpressionRef<'a>> {
    Some(match index {
        0 => call.get_prefix().into(),
        1 => call.get_arguments().into(),
        _ => return None,
    })
}

#[inline]
fn resolve_field_expression<'a>(
    index: usize,
    field: &'a FieldExpression,
) -> Option<AnyExpressionRef<'a>> {
    match index {
        0 => Some(field.get_prefix().into()),
        _ => None,
    }
}

#[inline]
fn resolve_index_expression<'a>(
    index: usize,
    index_expression: &'a IndexExpression,
) -> Option<AnyExpressionRef<'a>> {
    Some(match index {
        0 => index_expression.get_prefix().into(),
        1 => index_expression.get_index().into(),
        _ => return None,
    })
}

#[inline]
fn resolve_parentheses_expression<'a>(
    index: usize,
    parentheses: &'a ParentheseExpression,
) -> Option<AnyExpressionRef<'a>> {
    match index {
        0 => Some(parentheses.inner_expression().into()),
        _ => None,
    }
}

#[inline]
fn resolve_table_expression<'a>(
    index: usize,
    table: &'a TableExpression,
) -> Option<AnyExpressionRef<'a>> {
    todo!()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatementPath {
    path: NodePath,
}

impl StatementPath {
    pub fn with_statement(mut self, index: usize) -> Self {
        self.path.push_statement(index);
        self
    }

    pub fn with_expression(mut self, index: usize) -> Self {
        self.path.push_expression(index);
        self
    }

    pub fn resolve<'a>(&self, block: &'a Block) -> Option<AnyStatementRef<'a>> {
        match self.path.resolve(block)? {
            AnyNodeRef::AnyStatement(statement) => Some(statement),
            AnyNodeRef::AnyExpression(_) => None,
        }
    }
}

impl FromStr for StatementPath {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: string.parse()?,
        })
    }
}

impl ToString for StatementPath {
    fn to_string(&self) -> String {
        self.path.to_string()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExpressionPath {
    path: NodePath,
}

impl ExpressionPath {
    pub fn with_statement(mut self, index: usize) -> Self {
        self.path.push_statement(index);
        self
    }

    pub fn with_expression(mut self, index: usize) -> Self {
        self.path.push_expression(index);
        self
    }

    pub fn resolve<'a>(&self, block: &'a Block) -> Option<AnyExpressionRef<'a>> {
        match self.path.resolve(block)? {
            AnyNodeRef::AnyStatement(_) => None,
            AnyNodeRef::AnyExpression(expression) => Some(expression),
        }
    }
}

impl FromStr for ExpressionPath {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: string.parse()?,
        })
    }
}

impl ToString for ExpressionPath {
    fn to_string(&self) -> String {
        self.path.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct StatementSpan {}

#[derive(Clone, Debug)]
pub enum StatementMutation {
    Remove(StatementSpan),
    Replace(StatementSpan, Vec<Statement>),
    Insert(StatementPath, Vec<Statement>),
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_statement_paths {
        ($($name:ident ( $path:expr, $code:literal ) => $expect_code:literal ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let path = $path;

                    let parser = $crate::Parser::default();
                    let block = parser
                        .parse($code)
                        .expect("given test code should parse");

                    let expected_block = parser
                        .parse($expect_code)
                        .expect("given expected code should parse");

                    assert_eq!(expected_block.total_len(), 1);
                    let expected_statement = expected_block.get_statement(0).unwrap();

                    pretty_assertions::assert_eq!(
                        path.resolve(&block).expect("unable to resolve path"),
                        expected_statement
                    );
                }
            )*
        }
    }

    macro_rules! test_expression_paths {
        ($($name:ident ( $path:expr, $code:literal ) => $expect_code:literal ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let path = $path;

                    let parser = $crate::Parser::default();
                    let block = parser
                        .parse($code)
                        .expect("given test code should parse");

                    let expected_block = parser
                        .parse(&format!("return {}", $expect_code))
                        .expect("given expected code should parse");

                    assert_eq!(expected_block.total_len(), 1);
                    let expected_expression = match expected_block
                        .get_last_statement()
                        .expect("expected block should have one return statement")
                    {
                        $crate::nodes::LastStatement::Return(statement) => {
                            assert_eq!(statement.len(), 1);
                            statement
                                .iter_expressions()
                                .next()
                                .expect("return statement should have one expression")
                        },
                        _ => panic!("return statement expected")
                    };

                    match path.resolve(&block).expect("unable to resolve path") {
                        AnyExpressionRef::Expression(expression) => {
                            pretty_assertions::assert_eq!(expression, expected_expression);
                        }
                        AnyExpressionRef::Prefix(prefix) => {
                            pretty_assertions::assert_eq!(
                                &Expression::from(prefix.clone()),
                                expected_expression,
                            );
                        }
                        AnyExpressionRef::Arguments(_) => {
                            panic!("unable to compare arguments using this test macro")
                        }
                    };
                }
            )*
        }
    }

    macro_rules! test_path_from_str {
        ($($name:ident ( $string:literal ) => $expected_path:expr ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let path: NodePath = $string
                        .parse()
                        .expect("unable to parse path");

                    pretty_assertions::assert_eq!(path, $expected_path);
                }
            )*
        }
    }

    macro_rules! test_path_strings {
        ($($name:ident => $path:literal ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let path: NodePath = $path
                        .parse()
                        .expect("unable to parse path");

                    let serialized = path.to_string();

                    pretty_assertions::assert_eq!($path, serialized);
                }
            )*
        }
    }

    mod statement_paths {
        use super::*;

        fn new() -> StatementPath {
            StatementPath::default()
        }

        test_statement_paths!(
            single_statement(
                new().with_statement(0),
                "do end"
            ) => "do end",
            second_statement(
                new().with_statement(1),
                "do end local var"
            ) => "local var",
            second_statement_is_last_statement(
                new().with_statement(1),
                "do end return 1"
            ) => "return 1",
            nested_local_definition(
                new().with_statement(0).with_statement(0),
                "do local a = 1 end"
            ) => "local a = 1",
            statement_with_nested_block(
                new().with_statement(0).with_statement(0),
                "do do while true do end end end"
            ) => "do while true do end end",
        );
    }

    mod expression_paths {
        use super::*;

        fn new() -> ExpressionPath {
            ExpressionPath::default()
        }

        test_expression_paths!(
            return_statement_first_value(
                new().with_statement(0).with_expression(0),
                "return true"
            ) => "true",
            return_statement_second_value(
                new().with_statement(0).with_expression(1),
                "return true, nil"
            ) => "nil",
            return_statement_left_of_binary(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return condition or value"
            ) => "condition",
            return_statement_right_of_binary(
                new().with_statement(0).with_expression(0).with_expression(1),
                "return condition or value"
            ) => "value",
            return_statement_binary(
                new().with_statement(0).with_expression(0),
                "return condition or value"
            ) => "condition or value",
            return_statement_value_in_parens(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return (var)"
            ) => "var",
            return_statement_value_in_unary(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return not var"
            ) => "var",
            return_statement_if_expression_condition(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return if condition then result else other"
            ) => "condition",
            return_statement_if_expression_result(
                new().with_statement(0).with_expression(0).with_expression(1),
                "return if condition then result else other"
            ) => "result",
            return_statement_if_expression_else_result(
                new().with_statement(0).with_expression(0).with_expression(2),
                "return if condition then result else other"
            ) => "other",
            return_statement_elseif_expression_condition(
                new().with_statement(0).with_expression(0).with_expression(2),
                "return if condition then result elseif condition2 then result2 else other"
            ) => "condition2",
            return_statement_elseif_expression_result(
                new().with_statement(0).with_expression(0).with_expression(3),
                "return if condition then result elseif condition2 then result2 else other"
            ) => "result2",
            return_statement_elseif_expression_else_result(
                new().with_statement(0).with_expression(0).with_expression(4),
                "return if condition then result elseif condition2 then result2 else other"
            ) => "other",
            return_statement_index_prefix(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return value[key]"
            ) => "value",
            return_statement_index_value(
                new().with_statement(0).with_expression(0).with_expression(1),
                "return value[key]"
            ) => "key",
            return_statement_field_prefix(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return object.key"
            ) => "object",
            return_statement_call_prefix(
                new().with_statement(0).with_expression(0).with_expression(0),
                "return object.callback()"
            ) => "object.callback",
            return_statement_call_first_argument(
                new().with_statement(0).with_expression(0).with_expression(1).with_expression(0),
                "return callback(true)"
            ) => "true",
            return_statement_call_second_argument(
                new().with_statement(0).with_expression(0).with_expression(1).with_expression(1),
                "return callback(1, 2)"
            ) => "2",
            local_assign_statement_first_value(
                new().with_statement(0).with_expression(0),
                "local condition = true"
            ) => "true",
            local_assign_statement_second_value(
                new().with_statement(0).with_expression(1),
                "local a, b = 1, 2"
            ) => "2",
            while_condition(
                new().with_statement(0).with_expression(0),
                "while condition do end"
            ) => "condition",
            repeat_condition(
                new().with_statement(0).with_expression(0),
                "repeat do end until condition == expected"
            ) => "condition == expected",
            statement_call_prefix(
                new().with_statement(0).with_expression(0),
                "print()"
            ) => "print",
            statement_call_first_argument(
                new().with_statement(0).with_expression(1).with_expression(0),
                "print('hello')"
            ) => "'hello'",
            statement_call_second_argument(
                new().with_statement(0).with_expression(1).with_expression(1),
                "print('variable', variable)"
            ) => "variable",
        );
    }

    mod path_from_string {
        use super::*;

        test_path_from_str!(
            statement_0("0/") => NodePath::default().with_statement(0),
            statement_1("1/") => NodePath::default().with_statement(1),
            statement_10("10/") => NodePath::default().with_statement(10),
            statement_004("004/") => NodePath::default().with_statement(4),
            statement_4_expr_1("4/1:") => NodePath::default().with_statement(4).with_expression(1),
            statement_4_expr_1_statement_0("4/1:0/")
                => NodePath::default().with_statement(4).with_expression(1).with_statement(0),
        );
    }

    mod path_strings {
        use super::*;

        test_path_strings!(
            statement_0 => "0/",
            statement_1 => "1/",
            statement_4  => "4/",
            statement_10  => "10/",
            statement_4_expr_1  => "4/1:",
            statement_4_expr_1_statement_0  => "4/1:0/",
        );
    }
}
