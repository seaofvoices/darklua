use std::{fmt, str::FromStr};

use full_moon::{
    ast::{self, Ast},
    tokenizer::{Symbol, TokenType},
};

use crate::nodes::{
    Arguments, AssignStatement, BinaryExpression, BinaryOperator, Block, CompoundAssignStatement,
    CompoundOperator, DoStatement, Expression, FieldExpression, FunctionCall, FunctionExpression,
    FunctionName, FunctionStatement, GenericForStatement, IfStatement, IndexExpression,
    LastStatement, LocalAssignStatement, LocalFunctionStatement, NumberExpression,
    NumericForStatement, Prefix, RepeatStatement, Statement, StringExpression, TableEntry,
    TableExpression, UnaryExpression, UnaryOperator, Variable, WhileStatement,
};

fn convert_prefix(prefix: &ast::Prefix) -> Result<Prefix, ConvertError> {
    Ok(match prefix {
        ast::Prefix::Expression(expression) => Prefix::Parenthese(convert_expression(expression)?),
        ast::Prefix::Name(name) => Prefix::Identifier(name.token().to_string()),
        _ => {
            return Err(ConvertError::Prefix {
                prefix: prefix.to_string(),
            })
        }
    })
}

fn convert_prefix_with_suffixes<'a>(
    prefix: &ast::Prefix,
    suffixes: impl Iterator<Item = &'a ast::Suffix>,
) -> Result<Prefix, ConvertError> {
    let mut prefix = convert_prefix(prefix)?;
    for suffix in suffixes {
        match suffix {
            ast::Suffix::Call(call_suffix) => match call_suffix {
                ast::Call::AnonymousCall(arguments) => {
                    prefix =
                        FunctionCall::new(prefix, convert_function_args(&arguments)?, None).into();
                }
                ast::Call::MethodCall(method_call) => {
                    prefix = FunctionCall::new(
                        prefix,
                        convert_function_args(method_call.args())?,
                        Some(method_call.name().token().to_string()),
                    )
                    .into();
                }
                _ => {
                    return Err(ConvertError::Call {
                        call: call_suffix.to_string(),
                    });
                }
            },
            ast::Suffix::Index(index) => match index {
                ast::Index::Brackets {
                    brackets: _,
                    expression,
                } => {
                    prefix = IndexExpression::new(prefix, convert_expression(&expression)?).into();
                }
                ast::Index::Dot { name, dot: _ } => {
                    prefix = FieldExpression::new(prefix, name.token().to_string()).into();
                }
                _ => {
                    return Err(ConvertError::Index {
                        index: index.to_string(),
                    });
                }
            },
            _ => {
                return Err(ConvertError::Suffix {
                    suffix: suffix.to_string(),
                });
            }
        }
    }
    Ok(prefix)
}

fn convert_function_args(args: &ast::FunctionArgs) -> Result<Arguments, ConvertError> {
    Ok(match args {
        ast::FunctionArgs::Parentheses {
            parentheses: _,
            arguments,
        } => {
            let values: Result<_, _> = arguments.iter().map(convert_expression).collect();
            Arguments::Tuple(values?)
        }
        ast::FunctionArgs::String(string) => StringExpression::new(string.token().to_string())
            .expect("unable to convert string expression")
            .into(),
        ast::FunctionArgs::TableConstructor(table) => convert_table(table)?.into(),
        _ => {
            return Err(ConvertError::FunctionArguments {
                arguments: args.to_string(),
            })
        }
    })
}

fn convert_statement(statement: &ast::Stmt) -> Result<Statement, ConvertError> {
    Ok(match statement {
        ast::Stmt::Assignment(assignment) => {
            let variables: Result<_, _> = assignment
                .variables()
                .iter()
                .map(convert_variable)
                .collect();
            let values: Result<_, _> = assignment
                .expressions()
                .iter()
                .map(convert_expression)
                .collect();
            AssignStatement::new(variables?, values?).into()
        }
        ast::Stmt::Do(do_statement) => {
            DoStatement::new(convert_block(do_statement.block())?).into()
        }
        ast::Stmt::FunctionCall(call) => {
            let prefix = convert_prefix_with_suffixes(call.prefix(), call.suffixes())?;
            match prefix {
                Prefix::Call(call) => call.into(),
                _ => panic!(
                    "FunctionCall should convert to a call statement, but got {:#?}",
                    prefix,
                ),
            }
        }
        ast::Stmt::FunctionDeclaration(declaration) => {
            let (block, parameters, is_variadic) = convert_function_body(declaration.body())?;
            let name = convert_function_name(declaration.name())?;
            FunctionStatement::new(name, block, parameters, is_variadic).into()
        }
        ast::Stmt::GenericFor(generic_for) => {
            let expressions: Result<_, _> = generic_for
                .expressions()
                .iter()
                .map(convert_expression)
                .collect();
            GenericForStatement::new(
                generic_for
                    .names()
                    .iter()
                    .map(|name| name.token().to_string())
                    .collect(),
                expressions?,
                convert_block(generic_for.block())?,
            )
            .into()
        }
        ast::Stmt::If(if_statement) => {
            let mut statement = IfStatement::create(
                convert_expression(if_statement.condition())?,
                convert_block(if_statement.block())?,
            );
            for else_if in if_statement.else_if().unwrap_or(&Vec::new()).iter() {
                statement.push_branch(
                    convert_expression(else_if.condition())?,
                    convert_block(else_if.block())?,
                );
            }
            if let Some(block) = if_statement.else_block() {
                statement.set_else_block(convert_block(block)?);
            }
            statement.into()
        }
        ast::Stmt::LocalAssignment(assignment) => {
            let variables = assignment
                .names()
                .iter()
                .map(|token_ref| token_ref.token().to_string())
                .collect();

            let values: Result<_, _> = assignment
                .expressions()
                .iter()
                .map(convert_expression)
                .collect();

            LocalAssignStatement::new(variables, values?).into()
        }
        ast::Stmt::LocalFunction(assignment) => {
            let (block, parameters, is_variadic) = convert_function_body(assignment.body())?;
            LocalFunctionStatement::new(
                assignment.name().token().to_string(),
                block,
                parameters,
                is_variadic,
            )
            .into()
        }
        ast::Stmt::NumericFor(numeric_for) => NumericForStatement::new(
            numeric_for.index_variable().token().to_string(),
            convert_expression(numeric_for.start())?,
            convert_expression(numeric_for.end())?,
            numeric_for.step().map(convert_expression).transpose()?,
            convert_block(numeric_for.block())?,
        )
        .into(),
        ast::Stmt::Repeat(repeat_statement) => RepeatStatement::new(
            convert_block(repeat_statement.block())?,
            convert_expression(repeat_statement.until())?,
        )
        .into(),
        ast::Stmt::While(while_statement) => WhileStatement::new(
            convert_block(while_statement.block())?,
            convert_expression(while_statement.condition())?,
        )
        .into(),
        ast::Stmt::CompoundAssignment(assignment) => CompoundAssignStatement::new(
            convert_compound_op(assignment.compound_operator())?,
            convert_variable(assignment.lhs())?,
            convert_expression(assignment.rhs())?,
        )
        .into(),
        ast::Stmt::ExportedTypeDeclaration(_) => {
            // todo!()
            DoStatement::default().into()
        }
        ast::Stmt::TypeDeclaration(_) => {
            // todo!()
            DoStatement::default().into()
        }
        _ => {
            return Err(ConvertError::Statement {
                statement: statement.to_string(),
            })
        }
    })
}

fn convert_last_statement(statement: &ast::LastStmt) -> Result<LastStatement, ConvertError> {
    Ok(match statement {
        ast::LastStmt::Break(_) => LastStatement::Break,
        ast::LastStmt::Continue(_) => LastStatement::Continue,
        ast::LastStmt::Return(statement) => {
            let values: Result<_, _> = statement.returns().iter().map(convert_expression).collect();
            LastStatement::Return(values?)
        }
        _ => {
            return Err(ConvertError::LastStatement {
                statement: statement.to_string(),
            })
        }
    })
}

fn convert_variable(variable: &ast::Var) -> Result<Variable, ConvertError> {
    Ok(match variable {
        ast::Var::Expression(var_expression) => {
            match convert_prefix_with_suffixes(var_expression.prefix(), var_expression.suffixes())?
            {
                Prefix::Identifier(name) => Variable::Identifier(name),
                Prefix::Field(field) => Variable::Field(field),
                Prefix::Index(index) => Variable::Index(index),
                Prefix::Call(_) | Prefix::Parenthese(_) => {
                    return Err(ConvertError::Variable {
                        variable: variable.to_string(),
                    })
                }
            }
        }
        ast::Var::Name(name) => Variable::Identifier(name.token().to_string()),
        _ => {
            return Err(ConvertError::Variable {
                variable: variable.to_string(),
            })
        }
    })
}

fn convert_function_name(name: &ast::FunctionName) -> Result<FunctionName, ConvertError> {
    let mut name_iter = name
        .names()
        .iter()
        .map(|token_ref| token_ref.token().to_string());
    Ok(FunctionName::new(
        name_iter.next().expect("should have at least one name"),
        name_iter.collect(),
        name.method_name()
            .map(|token_ref| token_ref.token().to_string()),
    ))
}

fn convert_expression(expression: &ast::Expression) -> Result<Expression, ConvertError> {
    Ok(match expression {
        ast::Expression::BinaryOperator { lhs, binop, rhs } => BinaryExpression::new(
            convert_binop(binop)?,
            convert_expression(lhs)?,
            convert_expression(rhs)?,
        )
        .into(),
        ast::Expression::Parentheses {
            contained: _,
            expression,
        } => convert_expression(expression)?.into(),
        ast::Expression::UnaryOperator { unop, expression } => {
            UnaryExpression::new(convert_unop(unop)?, convert_expression(expression)?).into()
        }
        ast::Expression::Value {
            value,
            type_assertion: _,
        } => match value.as_ref() {
            ast::Value::Function((_token, body)) => {
                let (block, parameters, is_variadic) = convert_function_body(body)?;
                FunctionExpression::new(block, parameters, is_variadic).into()
            }
            ast::Value::FunctionCall(call) => {
                let prefix = convert_prefix_with_suffixes(call.prefix(), call.suffixes())?;
                match prefix {
                    Prefix::Call(call) => call.into(),
                    _ => panic!(
                        "FunctionCall should convert to a Prefix, but got {:#?}",
                        prefix
                    ),
                }
            }
            ast::Value::TableConstructor(table) => convert_table(table)?.into(),
            ast::Value::Number(number) => NumberExpression::from_str(&number.token().to_string())
                .map_err(|err| ConvertError::Number {
                    number: number.to_string(),
                    parsing_error: err.to_string(),
                })?
                .into(),
            ast::Value::ParenthesesExpression(expression) => {
                Expression::Parenthese(convert_expression(expression)?.into())
            }
            ast::Value::String(token_ref) => StringExpression::new(token_ref.token().to_string())
                .expect("unable to convert string expression")
                .into(),
            ast::Value::Symbol(symbol) => match symbol.token().token_type() {
                TokenType::Symbol { symbol } => match symbol {
                    Symbol::True => Expression::True,
                    Symbol::False => Expression::False,
                    Symbol::Nil => Expression::Nil,
                    Symbol::Ellipse => Expression::VariableArguments,
                    _ => {
                        return Err(ConvertError::Expression {
                            expression: expression.to_string(),
                        })
                    }
                },
                _ => {
                    return Err(ConvertError::Expression {
                        expression: expression.to_string(),
                    })
                }
            },
            ast::Value::Var(var) => match var {
                ast::Var::Expression(var_expression) => convert_prefix_with_suffixes(
                    var_expression.prefix(),
                    var_expression.suffixes(),
                )?
                .into(),
                ast::Var::Name(token_ref) => Expression::Identifier(token_ref.token().to_string()),
                _ => {
                    return Err(ConvertError::Expression {
                        expression: expression.to_string(),
                    })
                }
            },
            _ => {
                return Err(ConvertError::Expression {
                    expression: expression.to_string(),
                })
            }
        },
        _ => {
            return Err(ConvertError::Expression {
                expression: expression.to_string(),
            })
        }
    })
}

fn convert_function_body(
    body: &ast::FunctionBody,
) -> Result<(Block, Vec<String>, bool), ConvertError> {
    let mut parameters = Vec::new();
    let mut is_variadic = false;
    for param in body.parameters().iter() {
        match param {
            ast::Parameter::Ellipse(_) => {
                if is_variadic {
                    return Err(ConvertError::FunctionParameters {
                        parameters: body.parameters().to_string(),
                    });
                } else {
                    is_variadic = true;
                }
            }
            ast::Parameter::Name(name) => {
                if is_variadic {
                    return Err(ConvertError::FunctionParameters {
                        parameters: body.parameters().to_string(),
                    });
                }
                parameters.push(name.token().to_string());
            }
            _ => {
                return Err(ConvertError::FunctionParameter {
                    parameter: param.to_string(),
                })
            }
        }
    }

    Ok((convert_block(body.block())?, parameters, is_variadic))
}

fn convert_table(table: &ast::TableConstructor) -> Result<TableExpression, ConvertError> {
    let entries: Result<_, _> = table.fields().iter().map(convert_table_entry).collect();
    Ok(TableExpression::new(entries?))
}

fn convert_table_entry(field: &ast::Field) -> Result<TableEntry, ConvertError> {
    Ok(match field {
        ast::Field::ExpressionKey {
            brackets: _,
            key,
            equal: _,
            value,
        } => TableEntry::Index(convert_expression(key)?, convert_expression(value)?),
        ast::Field::NameKey {
            key,
            equal: _,
            value,
        } => TableEntry::Field(key.token().to_string(), convert_expression(value)?),
        ast::Field::NoKey(value) => TableEntry::Value(convert_expression(value)?),
        _ => {
            return Err(ConvertError::TableEntry {
                entry: field.to_string(),
            })
        }
    })
}

fn convert_binop(operator: &ast::BinOp) -> Result<BinaryOperator, ConvertError> {
    Ok(match operator {
        ast::BinOp::And(_) => BinaryOperator::And,
        ast::BinOp::Caret(_) => BinaryOperator::Caret,
        ast::BinOp::GreaterThan(_) => BinaryOperator::GreaterThan,
        ast::BinOp::GreaterThanEqual(_) => BinaryOperator::GreaterOrEqualThan,
        ast::BinOp::LessThan(_) => BinaryOperator::LowerThan,
        ast::BinOp::LessThanEqual(_) => BinaryOperator::LowerOrEqualThan,
        ast::BinOp::Minus(_) => BinaryOperator::Minus,
        ast::BinOp::Or(_) => BinaryOperator::Or,
        ast::BinOp::Percent(_) => BinaryOperator::Percent,
        ast::BinOp::Plus(_) => BinaryOperator::Plus,
        ast::BinOp::Slash(_) => BinaryOperator::Slash,
        ast::BinOp::Star(_) => BinaryOperator::Asterisk,
        ast::BinOp::TildeEqual(_) => BinaryOperator::NotEqual,
        ast::BinOp::TwoDots(_) => BinaryOperator::Concat,
        ast::BinOp::TwoEqual(_) => BinaryOperator::Equal,
        _ => {
            return Err(ConvertError::BinaryOperator {
                operator: operator.to_string(),
            })
        }
    })
}

fn convert_unop(operator: &ast::UnOp) -> Result<UnaryOperator, ConvertError> {
    Ok(match operator {
        ast::UnOp::Minus(_) => UnaryOperator::Minus,
        ast::UnOp::Not(_) => UnaryOperator::Not,
        ast::UnOp::Hash(_) => UnaryOperator::Length,
        _ => {
            return Err(ConvertError::UnaryOperator {
                operator: operator.to_string(),
            })
        }
    })
}

fn convert_compound_op(
    operator: &ast::types::CompoundOp,
) -> Result<CompoundOperator, ConvertError> {
    Ok(match operator {
        ast::types::CompoundOp::PlusEqual(_) => CompoundOperator::Plus,
        ast::types::CompoundOp::MinusEqual(_) => CompoundOperator::Minus,
        ast::types::CompoundOp::StarEqual(_) => CompoundOperator::Asterisk,
        ast::types::CompoundOp::SlashEqual(_) => CompoundOperator::Slash,
        ast::types::CompoundOp::PercentEqual(_) => CompoundOperator::Percent,
        ast::types::CompoundOp::CaretEqual(_) => CompoundOperator::Caret,
        ast::types::CompoundOp::TwoDotsEqual(_) => CompoundOperator::Concat,
        _ => {
            return Err(ConvertError::CompoundOperator {
                operator: operator.to_string(),
            })
        }
    })
}

fn convert_block(block: &ast::Block) -> Result<Block, ConvertError> {
    let statements: Result<_, _> = block.stmts().map(convert_statement).collect();
    Ok(Block::new(
        statements?,
        block.last_stmt().map(convert_last_statement).transpose()?,
    ))
}

fn convert_ast(ast: Ast) -> Result<Block, ConvertError> {
    convert_block(&ast.nodes())
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parser {}

impl Parser {
    pub fn parse(&self, code: &str) -> Result<Block, ParserError> {
        full_moon::parse(code)
            .map_err(ParserError::Parsing)
            .and_then(|ast| convert_ast(ast).map_err(ParserError::Converting))
    }
}

#[derive(Clone, Debug)]
pub enum ConvertError {
    Statement {
        statement: String,
    },
    LastStatement {
        statement: String,
    },
    Variable {
        variable: String,
    },
    FunctionArguments {
        arguments: String,
    },
    Call {
        call: String,
    },
    Index {
        index: String,
    },
    Suffix {
        suffix: String,
    },
    Prefix {
        prefix: String,
    },
    Number {
        number: String,
        parsing_error: String,
    },
    Expression {
        expression: String,
    },
    FunctionParameter {
        parameter: String,
    },
    FunctionParameters {
        parameters: String,
    },
    TableEntry {
        entry: String,
    },
    BinaryOperator {
        operator: String,
    },
    CompoundOperator {
        operator: String,
    },
    UnaryOperator {
        operator: String,
    },
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            _ => write!(f, "[AST conversion error]"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ParserError {
    Parsing(full_moon::Error),
    Converting(ConvertError),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parsing(err) => write!(f, "{}", err),
            Self::Converting(err) => write!(f, "{}", err),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_parse {
        ($($name:ident($input:literal) => $value:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let parser = Parser::default();
                    let block = parser.parse($input)
                        .expect(&format!("failed to parse `{}`", $input));

                    let expect_block = $value.into();
                    assert_eq!(block, expect_block);
                }
            )*
        };
    }

    test_parse!(
        empty_string("") => Block::default(),
        empty_do("do end") => DoStatement::default(),
        do_return_end("do return end") => DoStatement::new(LastStatement::Return(Vec::new()).into()),
        break_statement("break") => LastStatement::Break,
        return_no_values("return") => LastStatement::Return(Vec::new()),
        return_true("return true") => LastStatement::Return(vec![Expression::True]),
        return_false("return false") => LastStatement::Return(vec![Expression::False]),
        return_nil("return nil") => LastStatement::Return(vec![Expression::Nil]),
        return_variable_arguments("return ...") => LastStatement::Return(vec![Expression::VariableArguments]),
        return_variable("return var") => LastStatement::Return(vec![Expression::Identifier("var".to_owned())]),
        return_parentheses_true("return (true)") => LastStatement::Return(vec![
            Expression::Parenthese(Expression::True.into()),
        ]),
        return_true_false("return true, false") => LastStatement::Return(vec![
            Expression::True,
            Expression::False,
        ]),
        empty_while_true_do("while true do end") => WhileStatement::new(Block::default(), Expression::True),
        while_false_do_break("while false do break end") => WhileStatement::new(
            LastStatement::Break.into(),
            Expression::False,
        ),
        empty_repeat("repeat until true") => RepeatStatement::new(Block::default(), Expression::True),
        repeat_break("repeat break until true") => RepeatStatement::new(
            LastStatement::Break.into(),
            Expression::True,
        ),
        local_assignment_with_no_values("local var") => LocalAssignStatement::from_variable("var"),
        multiple_local_assignment_with_no_values("local foo, bar") => LocalAssignStatement::from_variable("foo")
            .with_variable("bar"),
        local_assignment_with_one_value("local var = true") => LocalAssignStatement::from_variable("var")
        .with_value(Expression::True),
        multiple_local_assignment_with_two_values("local foo, bar = true, false") => LocalAssignStatement::from_variable("foo")
            .with_variable("bar")
            .with_value(Expression::True)
            .with_value(Expression::False),
        return_binary_and("return true and false") => LastStatement::Return(vec![
            BinaryExpression::new(BinaryOperator::And, Expression::True, Expression::False).into(),
        ]),
        return_zero("return 0") => LastStatement::Return(vec![
            NumberExpression::from_str("0").unwrap().into(),
        ]),
        return_one("return 1") => LastStatement::Return(vec![
            NumberExpression::from_str("1").unwrap().into(),
        ]),
        return_float("return 1.5") => LastStatement::Return(vec![
            NumberExpression::from_str("1.5").unwrap().into(),
        ]),
        return_zero_point_five("return .5") => LastStatement::Return(vec![
            NumberExpression::from_str(".5").unwrap().into(),
        ]),
        return_not_true("return not true") => LastStatement::Return(vec![
            UnaryExpression::new(UnaryOperator::Not, Expression::True).into(),
        ]),
        return_variable_length("return #array") => LastStatement::Return(vec![
            UnaryExpression::new(
                UnaryOperator::Length,
                Expression::Identifier("array".to_owned()),
            ).into(),
        ]),
        return_minus_variable("return -num") => LastStatement::Return(vec![
            UnaryExpression::new(
                UnaryOperator::Minus,
                Expression::Identifier("num".to_owned()),
            ).into(),
        ]),
        call_function("call()") => FunctionCall::from_name("call"),
        call_indexed_table("foo.bar()") => FunctionCall::from_prefix(
            FieldExpression::new(Prefix::from_name("foo"), "bar")
        ),
        call_method("foo:bar()") => FunctionCall::from_name("foo").with_method("bar"),
        call_method_with_one_argument("foo:bar(true)") => FunctionCall::from_name("foo")
            .with_method("bar")
            .append_argument(Expression::True),
        call_function_with_one_argument("call(true)") => FunctionCall::from_name("call")
            .append_argument(Expression::True),
        call_function_with_two_arguments("call(true, false)") => FunctionCall::from_name("call")
            .append_argument(Expression::True)
            .append_argument(Expression::False),
        call_chain_empty("call()()") => FunctionCall::from_prefix(
            FunctionCall::from_name("call")
        ),
        call_chain_with_args("call(true)(false)") => FunctionCall::from_prefix(
            FunctionCall::from_name("call").append_argument(Expression::True)
        ).append_argument(Expression::False),
        call_method_chain_empty("call():method()") => FunctionCall::from_prefix(
            FunctionCall::from_name("call")
        ).with_method("method"),
        call_method_chain_with_arguments("call(true):method(false)") => FunctionCall::from_prefix(
            FunctionCall::from_name("call").append_argument(Expression::True)
        ).with_method("method").append_argument(Expression::False),
        call_index_chain_empty("call().method()") => FunctionCall::from_prefix(
            FieldExpression::new(FunctionCall::from_name("call"), "method")
        ),
        call_with_empty_table_argument("call{}") => FunctionCall::from_name("call")
            .with_arguments(TableExpression::default()),
        call_with_empty_string_argument("call''") => FunctionCall::from_name("call")
            .with_arguments(StringExpression::empty()),
        return_call_function("return call()") => LastStatement::Return(vec![
            FunctionCall::from_name("call").into(),
        ]),
        return_call_indexed_table("return foo.bar()") => LastStatement::Return(vec![
            FunctionCall::from_prefix(FieldExpression::new(Prefix::from_name("foo"), "bar")).into(),
        ]),
        return_call_method("return foo:bar()") => LastStatement::Return(vec![
            FunctionCall::from_name("foo").with_method("bar").into(),
        ]),
        return_call_method_with_one_argument("return foo:bar(true)") => LastStatement::Return(vec![
            FunctionCall::from_name("foo").with_method("bar").append_argument(Expression::True).into(),
        ]),
        return_call_function_with_one_argument("return call(true)") => LastStatement::Return(vec![
            FunctionCall::from_name("call").append_argument(Expression::True).into(),
        ]),
        return_call_function_with_two_arguments("return call(true, false)") => LastStatement::Return(vec![
            FunctionCall::from_name("call")
                .append_argument(Expression::True)
                .append_argument(Expression::False)
                .into(),
        ]),
        return_call_chain_empty("return call()()") => LastStatement::Return(vec![
            FunctionCall::from_prefix(FunctionCall::from_name("call")).into(),
        ]),
        return_call_chain_with_args("return call(true)(false)") => LastStatement::Return(vec![
            FunctionCall::from_prefix(
                FunctionCall::from_name("call").append_argument(Expression::True)
            ).append_argument(Expression::False).into(),
        ]),
        return_call_method_chain_empty("return call():method()") => LastStatement::Return(vec![
            FunctionCall::from_prefix(FunctionCall::from_name("call")).with_method("method").into(),
        ]),
        return_call_method_chain_with_arguments("return call(true):method(false)")
            => LastStatement::Return(vec![
                FunctionCall::from_prefix(FunctionCall::from_name("call").append_argument(Expression::True))
                    .with_method("method")
                    .append_argument(Expression::False)
                    .into(),
            ]),
        return_call_index_chain_empty("return call().method()") => LastStatement::Return(vec![
            FunctionCall::from_prefix(FieldExpression::new(FunctionCall::from_name("call"), "method")).into(),
        ]),
        return_call_new_empty_function("return (function() end)()") => LastStatement::Return(vec![
            FunctionCall::from_prefix(
                Prefix::Parenthese(FunctionExpression::default().into())
            ).into(),
        ]),
        return_call_variable_argument("return (...)()") => LastStatement::Return(vec![
            FunctionCall::from_prefix(Prefix::Parenthese(Expression::VariableArguments)).into(),
        ]),
        return_call_variable_in_parentheses("return (var)()") => LastStatement::Return(vec![
            FunctionCall::from_prefix(Prefix::Parenthese(Expression::identifier("var"))).into(),
        ]),
        return_call_variable_in_double_parentheses("return ((var))()") => LastStatement::Return(vec![
            FunctionCall::from_prefix(
                Prefix::Parenthese(Expression::identifier("var").in_parentheses())
            ).into(),
        ]),
        return_field_expression("return math.huge") => LastStatement::Return(vec![
            FieldExpression::new(Prefix::from_name("math"), "huge").into()
        ]),
        index_field_function_call("return call().result") => LastStatement::Return(vec![
            FieldExpression::new(FunctionCall::from_name("call"), "result").into(),
        ]),
        return_index_expression("return value[true]") => LastStatement::Return(vec![
            IndexExpression::new(Prefix::from_name("value"), Expression::True).into()
        ]),
        return_empty_table("return {}") => LastStatement::Return(vec![
            TableExpression::default().into()
        ]),
        return_array_with_one_element("return {true}") => LastStatement::Return(vec![
            TableExpression::default().append_array_value(Expression::True).into()
        ]),
        return_array_with_two_elements("return {true, false}") => LastStatement::Return(vec![
            TableExpression::default()
                .append_array_value(Expression::True)
                .append_array_value(Expression::False)
                .into()
        ]),
        return_array_with_one_field("return { field = true }") => LastStatement::Return(vec![
            TableExpression::default().append_field("field", Expression::True).into()
        ]),
        return_array_with_one_key_expression("return { [false] = true }") => LastStatement::Return(vec![
            TableExpression::default().append_index(Expression::False, Expression::True).into()
        ]),
        assign_variable("var = true") => AssignStatement::from_variable(
            Variable::new("var"),
            Expression::True,
        ),
        assign_two_variables("var, var2 = true, false") => AssignStatement::from_variable(
            Variable::new("var"),
            Expression::True,
        ).append_assignment(Variable::new("var2"), Expression::False),
        assign_field("var.field = true") => AssignStatement::from_variable(
            FieldExpression::new(Prefix::from_name("var"), "field"),
            Expression::True,
        ),
        assign_index("var[false] = true") => AssignStatement::from_variable(
            IndexExpression::new(Prefix::from_name("var"), Expression::False),
            Expression::True,
        ),
        return_empty_function("return function() end") => LastStatement::Return(vec![
            FunctionExpression::default().into(),
        ]),
        return_empty_function_with_one_param("return function(a) end") => LastStatement::Return(vec![
            FunctionExpression::default().with_parameter("a").into(),
        ]),
        return_empty_function_with_two_params("return function(a, b) end") => LastStatement::Return(vec![
            FunctionExpression::default().with_parameter("a").with_parameter("b").into(),
        ]),
        return_empty_variadic_function("return function(...) end") => LastStatement::Return(vec![
            FunctionExpression::default().variadic().into(),
        ]),
        return_empty_variadic_function_with_one_param("return function(a, ...) end")
            => LastStatement::Return(vec![
                FunctionExpression::default().with_parameter("a").variadic().into(),
            ]),
        return_function_that_returns("return function() return true end")
            => LastStatement::Return(vec![
                FunctionExpression::from_block(LastStatement::Return(vec![Expression::True])).into(),
            ]),
        empty_if_statement("if true then end") => IfStatement::create(Expression::True, Block::default()),
        if_statement_returns("if true then return end") => IfStatement::create(
            Expression::True,
            LastStatement::Return(Vec::new()).into(),
        ),
        empty_if_statement_with_empty_else("if true then else end")
            => IfStatement::create(Expression::True, Block::default())
                .with_else_block(Block::default()),
        empty_if_statement_with_empty_elseif("if true then elseif false then end")
            => IfStatement::create(Expression::True, Block::default())
                .with_branch(Expression::False, Block::default()),
        empty_if_statement_with_empty_elseif_and_empty_else("if true then elseif false then else end")
            => IfStatement::create(Expression::True, Block::default())
                .with_branch(Expression::False, Block::default())
                .with_else_block(Block::default()),
        empty_if_statement_with_returning_else("if true then else return end")
            => IfStatement::create(Expression::True, Block::default())
                .with_else_block(LastStatement::Return(Vec::new()).into()),
        empty_local_function("local function name() end")
            => LocalFunctionStatement::from_name("name", Block::default()),
        empty_local_function_variadic("local function name(...) end")
            => LocalFunctionStatement::from_name("name", Block::default()).variadic(),
        empty_local_function_variadic_with_one_parameter("local function name(a, ...) end")
            => LocalFunctionStatement::from_name("name", Block::default())
                .with_parameter("a")
                .variadic(),
        local_function_return("local function name() return end")
            => LocalFunctionStatement::from_name("name", LastStatement::Return(Vec::new())),

        empty_function_statement("function name() end")
            => FunctionStatement::from_name("name", Block::default()),
        empty_function_statement_variadic("function name(...) end")
            => FunctionStatement::from_name("name", Block::default()).variadic(),
        empty_function_statement_variadic_with_one_parameter("function name(a, ...) end")
            => FunctionStatement::from_name("name", Block::default())
                .with_parameter("a")
                .variadic(),
        function_statement_return("function name() return end")
            => FunctionStatement::from_name("name", LastStatement::Return(Vec::new())),
        empty_generic_for("for key in pairs(t) do end") => GenericForStatement::new(
            vec!["key".to_owned()],
            vec![
                FunctionCall::from_name("pairs")
                    .append_argument(Expression::identifier("t"))
                    .into(),
            ],
            Block::default(),
        ),
        empty_generic_for_multiple_variables("for key, value in pairs(t) do end") => GenericForStatement::new(
            vec!["key".to_owned(), "value".to_owned()],
            vec![
                FunctionCall::from_name("pairs")
                    .append_argument(Expression::identifier("t"))
                    .into(),
            ],
            Block::default(),
        ),
        empty_generic_for_multiple_values("for key in next, t do end") => GenericForStatement::new(
            vec!["key".to_owned()],
            vec![Expression::identifier("next"), Expression::identifier("t")],
            Block::default(),
        ),
        generic_for_break("for key in pairs(t) do break end") => GenericForStatement::new(
            vec!["key".to_owned()],
            vec![
                FunctionCall::from_name("pairs")
                    .append_argument(Expression::identifier("t"))
                    .into(),
            ],
            LastStatement::Break,
        ),
        empty_numeric_for("for i=start, bound do end") => NumericForStatement::new(
            "i",
            Expression::identifier("start"),
            Expression::identifier("bound"),
            None,
            Block::default(),
        ),
        empty_numeric_for_with_step("for i=start, bound, step do end") => NumericForStatement::new(
            "i",
            Expression::identifier("start"),
            Expression::identifier("bound"),
            Some(Expression::identifier("step")),
            Block::default(),
        ),
        numeric_for_that_breaks("for i=start, bound do break end") => NumericForStatement::new(
            "i",
            Expression::identifier("start"),
            Expression::identifier("bound"),
            None,
            LastStatement::Break,
        ),
    );
}
