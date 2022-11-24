use std::{fmt, str::FromStr};

use full_moon::{
    ast::{self, Ast},
    tokenizer::{self, Symbol, TokenType},
};

use crate::nodes::*;

fn get_binary_operator_token(
    operator: &ast::BinOp,
) -> Result<&tokenizer::TokenReference, ConvertError> {
    use ast::BinOp;

    match operator {
        BinOp::And(token)
        | BinOp::Caret(token)
        | BinOp::GreaterThan(token)
        | BinOp::GreaterThanEqual(token)
        | BinOp::LessThan(token)
        | BinOp::LessThanEqual(token)
        | BinOp::Minus(token)
        | BinOp::Or(token)
        | BinOp::Percent(token)
        | BinOp::Plus(token)
        | BinOp::Slash(token)
        | BinOp::Star(token)
        | BinOp::TildeEqual(token)
        | BinOp::TwoDots(token)
        | BinOp::TwoEqual(token) => Ok(token),
        _ => Err(ConvertError::CompoundOperator {
            operator: operator.to_string(),
        }),
    }
}

fn get_unary_operator_token(
    operator: &ast::UnOp,
) -> Result<&tokenizer::TokenReference, ConvertError> {
    use ast::UnOp;

    match operator {
        UnOp::Minus(token) | UnOp::Not(token) | UnOp::Hash(token) => Ok(token),
        _ => Err(ConvertError::CompoundOperator {
            operator: operator.to_string(),
        }),
    }
}

fn get_compound_operator_token(
    operator: &ast::types::CompoundOp,
) -> Result<&tokenizer::TokenReference, ConvertError> {
    use ast::types::CompoundOp;

    match operator {
        CompoundOp::PlusEqual(token)
        | CompoundOp::MinusEqual(token)
        | CompoundOp::StarEqual(token)
        | CompoundOp::SlashEqual(token)
        | CompoundOp::PercentEqual(token)
        | CompoundOp::CaretEqual(token)
        | CompoundOp::TwoDotsEqual(token) => Ok(token),
        _ => Err(ConvertError::CompoundOperator {
            operator: operator.to_string(),
        }),
    }
}

#[derive(Debug)]
struct FunctionBodyTokens {
    pub opening_parenthese: Token,
    pub closing_parenthese: Token,
    pub end: Token,
    pub parameter_commas: Vec<Token>,
    pub variable_arguments: Option<Token>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parser {
    hold_token_data: bool,
}

impl Parser {
    pub fn parse(&self, code: &str) -> Result<Block, ParserError> {
        full_moon::parse(code)
            .map_err(ParserError::Parsing)
            .and_then(|ast| self.convert_ast(ast).map_err(ParserError::Converting))
    }

    pub fn preserve_tokens(mut self) -> Self {
        self.hold_token_data = true;
        self
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_prefix(&self, prefix: &ast::Prefix) -> Result<Prefix, ConvertError> {
        Ok(match prefix {
            ast::Prefix::Expression(expression) => match self.convert_expression(expression)? {
                Expression::Parenthese(parenthese) => Prefix::Parenthese(*parenthese),
                _ => {
                    return Err(ConvertError::Prefix {
                        prefix: prefix.to_string(),
                    })
                }
            },
            ast::Prefix::Name(name) => self.convert_token_to_identifier(name).into(),
            _ => {
                return Err(ConvertError::Prefix {
                    prefix: prefix.to_string(),
                })
            }
        })
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_prefix_with_suffixes<'a>(
        &self,
        prefix: &ast::Prefix,
        suffixes: impl Iterator<Item = &'a ast::Suffix>,
    ) -> Result<Prefix, ConvertError> {
        let mut prefix = self.convert_prefix(prefix)?;
        for suffix in suffixes {
            match suffix {
                ast::Suffix::Call(call_suffix) => match call_suffix {
                    ast::Call::AnonymousCall(arguments) => {
                        let mut call =
                            FunctionCall::new(prefix, self.convert_function_args(arguments)?, None);
                        if self.hold_token_data {
                            call.set_tokens(FunctionCallTokens { colon: None })
                        }
                        prefix = call.into();
                    }
                    ast::Call::MethodCall(method_call) => {
                        let mut call = FunctionCall::new(
                            prefix,
                            self.convert_function_args(method_call.args())?,
                            Some(self.convert_token_to_identifier(method_call.name())),
                        );
                        if self.hold_token_data {
                            call.set_tokens(FunctionCallTokens {
                                colon: Some(self.convert_token(method_call.colon_token())),
                            });
                        }
                        prefix = call.into();
                    }
                    _ => {
                        return Err(ConvertError::Call {
                            call: call_suffix.to_string(),
                        });
                    }
                },
                ast::Suffix::Index(index) => match index {
                    ast::Index::Brackets {
                        brackets,
                        expression,
                    } => {
                        let mut index =
                            IndexExpression::new(prefix, self.convert_expression(expression)?);
                        if self.hold_token_data {
                            let (left, right) = brackets.tokens();
                            index.set_tokens(IndexExpressionTokens {
                                opening_bracket: self.convert_token(left),
                                closing_bracket: self.convert_token(right),
                            });
                        }
                        prefix = index.into();
                    }
                    ast::Index::Dot { name, dot } => {
                        let mut field =
                            FieldExpression::new(prefix, self.convert_token_to_identifier(name));
                        if self.hold_token_data {
                            field.set_token(self.convert_token(dot));
                        }
                        prefix = field.into();
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

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_function_args(&self, args: &ast::FunctionArgs) -> Result<Arguments, ConvertError> {
        Ok(match args {
            ast::FunctionArgs::Parentheses {
                parentheses,
                arguments,
            } => {
                let values: Result<_, _> = arguments
                    .iter()
                    .map(|expression| self.convert_expression(expression))
                    .collect();
                let mut tuple = TupleArguments::new(values?);
                if self.hold_token_data {
                    let (left, right) = parentheses.tokens();
                    tuple.set_tokens(TupleArgumentsTokens {
                        opening_parenthese: self.convert_token(left),
                        closing_parenthese: self.convert_token(right),
                        commas: self.extract_tokens_from_punctuation(arguments),
                    })
                }
                tuple.into()
            }
            ast::FunctionArgs::String(string) => self.convert_string_expression(string).into(),
            ast::FunctionArgs::TableConstructor(table) => self.convert_table(table)?.into(),
            _ => {
                return Err(ConvertError::FunctionArguments {
                    arguments: args.to_string(),
                })
            }
        })
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_string_expression(&self, string: &tokenizer::TokenReference) -> StringExpression {
        let mut expression = StringExpression::new(&string.token().to_string())
            .expect("unable to convert string expression");
        if self.hold_token_data {
            expression.set_token(self.convert_token(string));
        }
        expression
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_statement(&self, statement: &ast::Stmt) -> Result<Statement, ConvertError> {
        Ok(match statement {
            ast::Stmt::Assignment(assignment) => {
                let variables: Result<_, _> = assignment
                    .variables()
                    .iter()
                    .map(|variable| self.convert_variable(variable))
                    .collect();
                let values: Result<_, _> = assignment
                    .expressions()
                    .iter()
                    .map(|expression| self.convert_expression(expression))
                    .collect();
                let mut statement = AssignStatement::new(variables?, values?);
                if self.hold_token_data {
                    statement.set_tokens(AssignTokens {
                        equal: self.convert_token(assignment.equal_token()),
                        variable_commas: self
                            .extract_tokens_from_punctuation(assignment.variables()),
                        value_commas: self
                            .extract_tokens_from_punctuation(assignment.expressions()),
                    });
                }
                statement.into()
            }
            ast::Stmt::Do(do_statement) => {
                let mut statement = DoStatement::new(self.convert_block(do_statement.block())?);
                if self.hold_token_data {
                    statement.set_tokens(DoTokens {
                        r#do: self.convert_token(do_statement.do_token()),
                        end: self.convert_token(do_statement.end_token()),
                    })
                }
                statement.into()
            }
            ast::Stmt::FunctionCall(call) => {
                let prefix = self.convert_prefix_with_suffixes(call.prefix(), call.suffixes())?;
                match prefix {
                    Prefix::Call(call) => call.into(),
                    _ => panic!(
                        "FunctionCall should convert to a call statement, but got {:#?}",
                        prefix,
                    ),
                }
            }
            ast::Stmt::FunctionDeclaration(declaration) => {
                let (block, parameters, is_variadic, tokens) =
                    self.convert_function_body(declaration.body())?;
                let name = self.convert_function_name(declaration.name())?;
                let mut function = FunctionStatement::new(name, block, parameters, is_variadic);

                if let Some(tokens) = tokens {
                    function.set_tokens(FunctionStatementTokens {
                        function: self.convert_token(declaration.function_token()),
                        opening_parenthese: tokens.opening_parenthese,
                        closing_parenthese: tokens.closing_parenthese,
                        end: tokens.end,
                        parameter_commas: tokens.parameter_commas,
                        variable_arguments: tokens.variable_arguments,
                    });
                }

                function.into()
            }
            ast::Stmt::GenericFor(generic_for) => {
                let expressions: Result<_, _> = generic_for
                    .expressions()
                    .iter()
                    .map(|expression| self.convert_expression(expression))
                    .collect();
                let mut generic = GenericForStatement::new(
                    generic_for
                        .names()
                        .iter()
                        .map(|name| self.convert_token_to_identifier(name))
                        .collect(),
                    expressions?,
                    self.convert_block(generic_for.block())?,
                );
                if self.hold_token_data {
                    generic.set_tokens(GenericForTokens {
                        r#for: self.convert_token(generic_for.for_token()),
                        r#in: self.convert_token(generic_for.in_token()),
                        r#do: self.convert_token(generic_for.do_token()),
                        end: self.convert_token(generic_for.end_token()),
                        identifier_commas: self
                            .extract_tokens_from_punctuation(generic_for.names()),
                        value_commas: self
                            .extract_tokens_from_punctuation(generic_for.expressions()),
                    });
                }
                generic.into()
            }
            ast::Stmt::If(if_statement) => {
                let mut statement = IfStatement::create(
                    self.convert_expression(if_statement.condition())?,
                    self.convert_block(if_statement.block())?,
                );
                for else_if in if_statement.else_if().unwrap_or(&Vec::new()).iter() {
                    let mut branch = IfBranch::new(
                        self.convert_expression(else_if.condition())?,
                        self.convert_block(else_if.block())?,
                    );
                    if self.hold_token_data {
                        branch.set_tokens(IfBranchTokens {
                            elseif: self.convert_token(else_if.else_if_token()),
                            then: self.convert_token(else_if.then_token()),
                        });
                    }
                    statement.push_branch(branch);
                }
                if let Some(block) = if_statement.else_block() {
                    statement.set_else_block(self.convert_block(block)?);
                }
                if self.hold_token_data {
                    statement.set_tokens(IfStatementTokens {
                        r#if: self.convert_token(if_statement.if_token()),
                        then: self.convert_token(if_statement.then_token()),
                        end: self.convert_token(if_statement.end_token()),
                        r#else: if_statement
                            .else_token()
                            .map(|token| self.convert_token(token)),
                    })
                }
                statement.into()
            }
            ast::Stmt::LocalAssignment(assignment) => {
                let variables = assignment
                    .names()
                    .iter()
                    .map(|token_ref| self.convert_token_to_identifier(token_ref))
                    .collect();

                let values: Result<_, _> = assignment
                    .expressions()
                    .iter()
                    .map(|expression| self.convert_expression(expression))
                    .collect();

                let mut statement = LocalAssignStatement::new(variables, values?);

                if self.hold_token_data {
                    statement.set_tokens(LocalAssignTokens {
                        local: self.convert_token(assignment.local_token()),
                        equal: assignment
                            .equal_token()
                            .map(|token| self.convert_token(token)),
                        variable_commas: self.extract_tokens_from_punctuation(assignment.names()),
                        value_commas: self
                            .extract_tokens_from_punctuation(assignment.expressions()),
                    })
                }

                statement.into()
            }
            ast::Stmt::LocalFunction(assignment) => {
                let (block, parameters, is_variadic, tokens) =
                    self.convert_function_body(assignment.body())?;
                let mut name = Identifier::new(assignment.name().token().to_string());
                if self.hold_token_data {
                    name.set_token(self.convert_token(assignment.name()));
                }
                let mut statement =
                    LocalFunctionStatement::new(name, block, parameters, is_variadic);
                if let Some(tokens) = tokens {
                    statement.set_tokens(LocalFunctionTokens {
                        local: self.convert_token(assignment.local_token()),
                        function: self.convert_token(assignment.function_token()),
                        opening_parenthese: tokens.opening_parenthese,
                        closing_parenthese: tokens.closing_parenthese,
                        end: tokens.end,
                        parameter_commas: tokens.parameter_commas,
                        variable_arguments: tokens.variable_arguments,
                    });
                }
                statement.into()
            }
            ast::Stmt::NumericFor(numeric_for) => {
                let mut statement = NumericForStatement::new(
                    self.convert_token_to_identifier(numeric_for.index_variable()),
                    self.convert_expression(numeric_for.start())?,
                    self.convert_expression(numeric_for.end())?,
                    numeric_for
                        .step()
                        .map(|expression| self.convert_expression(expression))
                        .transpose()?,
                    self.convert_block(numeric_for.block())?,
                );
                if self.hold_token_data {
                    statement.set_tokens(NumericForTokens {
                        r#for: self.convert_token(numeric_for.for_token()),
                        equal: self.convert_token(numeric_for.equal_token()),
                        r#do: self.convert_token(numeric_for.do_token()),
                        end: self.convert_token(numeric_for.end_token()),
                        end_comma: self.convert_token(numeric_for.start_end_comma()),
                        step_comma: numeric_for
                            .end_step_comma()
                            .map(|token| self.convert_token(token)),
                    });
                }
                statement.into()
            }
            ast::Stmt::Repeat(repeat_statement) => {
                let mut statement = RepeatStatement::new(
                    self.convert_block(repeat_statement.block())?,
                    self.convert_expression(repeat_statement.until())?,
                );
                if self.hold_token_data {
                    statement.set_tokens(RepeatTokens {
                        repeat: self.convert_token(repeat_statement.repeat_token()),
                        until: self.convert_token(repeat_statement.until_token()),
                    });
                }
                statement.into()
            }
            ast::Stmt::While(while_statement) => {
                let mut statement = WhileStatement::new(
                    self.convert_block(while_statement.block())?,
                    self.convert_expression(while_statement.condition())?,
                );
                if self.hold_token_data {
                    statement.set_tokens(WhileTokens {
                        r#while: self.convert_token(while_statement.while_token()),
                        r#do: self.convert_token(while_statement.do_token()),
                        end: self.convert_token(while_statement.end_token()),
                    });
                }
                statement.into()
            }
            ast::Stmt::CompoundAssignment(assignment) => {
                let mut statement = CompoundAssignStatement::new(
                    self.convert_compound_op(assignment.compound_operator())?,
                    self.convert_variable(assignment.lhs())?,
                    self.convert_expression(assignment.rhs())?,
                );
                if self.hold_token_data {
                    statement.set_tokens(CompoundAssignTokens {
                        operator: self.convert_token(get_compound_operator_token(
                            assignment.compound_operator(),
                        )?),
                    });
                }
                statement.into()
            }
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

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_token(&self, token: &tokenizer::TokenReference) -> Token {
        let mut new_token = Token::new_with_line(
            token.start_position().bytes(),
            token.end_position().bytes(),
            token.start_position().line(),
        );

        for trivia_token in token.leading_trivia() {
            new_token.push_leading_trivia(self.convert_trivia(trivia_token));
        }

        for trivia_token in token.trailing_trivia() {
            new_token.push_trailing_trivia(self.convert_trivia(trivia_token));
        }

        new_token
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_trivia(&self, token: &tokenizer::Token) -> Trivia {
        use tokenizer::TokenKind;

        match token.token_kind() {
            TokenKind::Eof => todo!(),
            TokenKind::Identifier => todo!(),
            TokenKind::MultiLineComment => TriviaKind::Comment,
            TokenKind::Number => todo!(),
            TokenKind::Shebang => todo!(),
            TokenKind::SingleLineComment => TriviaKind::Comment,
            TokenKind::StringLiteral => todo!(),
            TokenKind::Symbol => todo!(),
            TokenKind::Whitespace => TriviaKind::Whitespace,
            _ => todo!("unexpected token kind"),
        }
        .at(
            token.start_position().bytes(),
            token.end_position().bytes(),
            token.start_position().line(),
        )
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_token_to_identifier(&self, token: &tokenizer::TokenReference) -> Identifier {
        let mut identifier = Identifier::new(token.token().to_string());
        if self.hold_token_data {
            identifier.set_token(self.convert_token(token));
        }
        identifier
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn extract_tokens_from_punctuation<T>(
        &self,
        punctuated: &ast::punctuated::Punctuated<T>,
    ) -> Vec<Token> {
        punctuated
            .pairs()
            .filter_map(|pair| match pair {
                ast::punctuated::Pair::End(_) => None,
                ast::punctuated::Pair::Punctuated(_, token) => Some(self.convert_token(token)),
            })
            .collect()
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_last_statement(
        &self,
        statement: &ast::LastStmt,
    ) -> Result<LastStatement, ConvertError> {
        Ok(match statement {
            ast::LastStmt::Break(token) => {
                if self.hold_token_data {
                    LastStatement::Break(Some(self.convert_token(token)))
                } else {
                    LastStatement::new_break()
                }
            }
            ast::LastStmt::Continue(token) => {
                if self.hold_token_data {
                    LastStatement::Continue(Some(self.convert_token(token)))
                } else {
                    LastStatement::new_continue()
                }
            }
            ast::LastStmt::Return(statement) => {
                let values: Result<_, _> = statement
                    .returns()
                    .iter()
                    .map(|expression| self.convert_expression(expression))
                    .collect();
                let mut return_statement = ReturnStatement::new(values?);
                if self.hold_token_data {
                    let commas = self.extract_tokens_from_punctuation(statement.returns());
                    return_statement.set_tokens(ReturnTokens {
                        r#return: self.convert_token(statement.token()),
                        commas,
                    });
                }
                return_statement.into()
            }
            _ => {
                return Err(ConvertError::LastStatement {
                    statement: statement.to_string(),
                })
            }
        })
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_variable(&self, variable: &ast::Var) -> Result<Variable, ConvertError> {
        Ok(match variable {
            ast::Var::Expression(var_expression) => {
                match self.convert_prefix_with_suffixes(
                    var_expression.prefix(),
                    var_expression.suffixes(),
                )? {
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
            ast::Var::Name(name) => self.convert_token_to_identifier(name).into(),
            _ => {
                return Err(ConvertError::Variable {
                    variable: variable.to_string(),
                })
            }
        })
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_function_name(
        &self,
        name: &ast::FunctionName,
    ) -> Result<FunctionName, ConvertError> {
        let mut name_iter = name
            .names()
            .iter()
            .map(|token_ref| self.convert_token_to_identifier(token_ref));

        let mut function_name = FunctionName::new(
            name_iter.next().expect("should have at least one name"),
            name_iter.collect(),
            name.method_name()
                .map(|token_ref| self.convert_token_to_identifier(token_ref)),
        );

        if self.hold_token_data {
            function_name.set_tokens(FunctionNameTokens {
                periods: self.extract_tokens_from_punctuation(name.names()),
                colon: name.method_colon().map(|colon| self.convert_token(colon)),
            });
        }

        Ok(function_name)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_expression(&self, expression: &ast::Expression) -> Result<Expression, ConvertError> {
        Ok(match expression {
            ast::Expression::BinaryOperator { lhs, binop, rhs } => {
                let mut binary = BinaryExpression::new(
                    self.convert_binop(binop)?,
                    self.convert_expression(lhs)?,
                    self.convert_expression(rhs)?,
                );
                if self.hold_token_data {
                    binary.set_token(self.convert_token(get_binary_operator_token(binop)?));
                }
                binary.into()
            }
            ast::Expression::Parentheses {
                contained,
                expression: inner_expression,
            } => {
                let mut parenthese =
                    ParentheseExpression::new(self.convert_expression(inner_expression)?);
                if self.hold_token_data {
                    let (left, right) = contained.tokens();
                    parenthese.set_tokens(ParentheseTokens {
                        left_parenthese: self.convert_token(left),
                        right_parenthese: self.convert_token(right),
                    });
                }
                parenthese.into()
            }
            ast::Expression::UnaryOperator { unop, expression } => {
                let mut unary = UnaryExpression::new(
                    self.convert_unop(unop)?,
                    self.convert_expression(expression)?,
                );
                if self.hold_token_data {
                    unary.set_token(self.convert_token(get_unary_operator_token(unop)?));
                }
                unary.into()
            }
            ast::Expression::Value {
                value,
                type_assertion: _,
            } => match value.as_ref() {
                ast::Value::Function((token, body)) => {
                    let (block, parameters, is_variadic, tokens) =
                        self.convert_function_body(body)?;
                    let mut function = FunctionExpression::new(block, parameters, is_variadic);
                    if let Some(tokens) = tokens {
                        function.set_tokens(FunctionExpressionTokens {
                            function: self.convert_token(token),
                            opening_parenthese: tokens.opening_parenthese,
                            closing_parenthese: tokens.closing_parenthese,
                            end: tokens.end,
                            parameter_commas: tokens.parameter_commas,
                            variable_arguments: tokens.variable_arguments,
                        })
                    }
                    function.into()
                }
                ast::Value::FunctionCall(call) => {
                    let prefix =
                        self.convert_prefix_with_suffixes(call.prefix(), call.suffixes())?;
                    match prefix {
                        Prefix::Call(call) => call.into(),
                        _ => panic!(
                            "FunctionCall should convert to a Prefix, but got {:#?}",
                            prefix
                        ),
                    }
                }
                ast::Value::TableConstructor(table) => self.convert_table(table)?.into(),
                ast::Value::Number(number) => {
                    let mut expression = NumberExpression::from_str(&number.token().to_string())
                        .map_err(|err| ConvertError::Number {
                            number: number.to_string(),
                            parsing_error: err.to_string(),
                        })?;
                    if self.hold_token_data {
                        expression.set_token(self.convert_token(number));
                    }
                    expression.into()
                }
                ast::Value::ParenthesesExpression(expression) => {
                    self.convert_expression(expression)?
                }
                ast::Value::String(token_ref) => self.convert_string_expression(token_ref).into(),
                ast::Value::Symbol(symbol_token) => match symbol_token.token().token_type() {
                    TokenType::Symbol { symbol } => {
                        let token = if self.hold_token_data {
                            Some(self.convert_token(symbol_token))
                        } else {
                            None
                        };
                        match symbol {
                            Symbol::True => Expression::True(token),
                            Symbol::False => Expression::False(token),
                            Symbol::Nil => Expression::Nil(token),
                            Symbol::Ellipse => Expression::VariableArguments(token),
                            _ => {
                                return Err(ConvertError::Expression {
                                    expression: expression.to_string(),
                                })
                            }
                        }
                    }
                    _ => {
                        return Err(ConvertError::Expression {
                            expression: expression.to_string(),
                        })
                    }
                },
                ast::Value::Var(var) => match var {
                    ast::Var::Expression(var_expression) => self
                        .convert_prefix_with_suffixes(
                            var_expression.prefix(),
                            var_expression.suffixes(),
                        )?
                        .into(),
                    ast::Var::Name(token_ref) => {
                        Expression::Identifier(self.convert_token_to_identifier(token_ref))
                    }
                    _ => {
                        return Err(ConvertError::Expression {
                            expression: expression.to_string(),
                        })
                    }
                },
                ast::Value::IfExpression(if_expression) => {
                    let mut expression = IfExpression::new(
                        self.convert_expression(if_expression.condition())?,
                        self.convert_expression(if_expression.if_expression())?,
                        self.convert_expression(if_expression.else_expression())?,
                    );

                    if let Some(elseif_expressions) = if_expression.else_if_expressions() {
                        for elseif in elseif_expressions {
                            let mut branch = ElseIfExpressionBranch::new(
                                self.convert_expression(elseif.condition())?,
                                self.convert_expression(elseif.expression())?,
                            );
                            if self.hold_token_data {
                                branch.set_tokens(ElseIfExpressionBranchTokens {
                                    elseif: self.convert_token(elseif.else_if_token()),
                                    then: self.convert_token(elseif.then_token()),
                                });
                            }
                            expression.push_branch(branch);
                        }
                    }

                    if self.hold_token_data {
                        expression.set_tokens(IfExpressionTokens {
                            r#if: self.convert_token(if_expression.if_token()),
                            then: self.convert_token(if_expression.then_token()),
                            r#else: self.convert_token(if_expression.else_token()),
                        });
                    }

                    expression.into()
                }
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

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_function_body(
        &self,
        body: &ast::FunctionBody,
    ) -> Result<(Block, Vec<Identifier>, bool, Option<FunctionBodyTokens>), ConvertError> {
        let mut parameters = Vec::new();
        let mut is_variadic = None;
        for param in body.parameters().iter() {
            match param {
                ast::Parameter::Ellipse(token) => {
                    if is_variadic.is_some() {
                        return Err(ConvertError::FunctionParameters {
                            parameters: body.parameters().to_string(),
                        });
                    } else {
                        is_variadic = Some(token);
                    }
                }
                ast::Parameter::Name(name) => {
                    if is_variadic.is_some() {
                        return Err(ConvertError::FunctionParameters {
                            parameters: body.parameters().to_string(),
                        });
                    }
                    let mut identifier = Identifier::new(name.token().to_string());
                    if self.hold_token_data {
                        identifier.set_token(self.convert_token(name));
                    }
                    parameters.push(identifier);
                }
                _ => {
                    return Err(ConvertError::FunctionParameter {
                        parameter: param.to_string(),
                    })
                }
            }
        }

        let tokens = if self.hold_token_data {
            let (open, close) = body.parameters_parentheses().tokens();
            let commas = self.extract_tokens_from_punctuation(body.parameters());
            Some(FunctionBodyTokens {
                opening_parenthese: self.convert_token(open),
                closing_parenthese: self.convert_token(close),
                end: self.convert_token(body.end_token()),
                parameter_commas: commas,
                variable_arguments: is_variadic.map(|token| self.convert_token(token)),
            })
        } else {
            None
        };

        Ok((
            self.convert_block(body.block())?,
            parameters,
            is_variadic.is_some(),
            tokens,
        ))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_table(
        &self,
        table: &ast::TableConstructor,
    ) -> Result<TableExpression, ConvertError> {
        let entries: Result<_, _> = table
            .fields()
            .iter()
            .map(|entry| self.convert_table_entry(entry))
            .collect();
        let mut expression = TableExpression::new(entries?);
        if self.hold_token_data {
            let (left, right) = table.braces().tokens();
            expression.set_tokens(TableTokens {
                opening_brace: self.convert_token(left),
                closing_brace: self.convert_token(right),
                separators: self.extract_tokens_from_punctuation(table.fields()),
            });
        }
        Ok(expression)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_table_entry(&self, field: &ast::Field) -> Result<TableEntry, ConvertError> {
        Ok(match field {
            ast::Field::ExpressionKey {
                brackets,
                key,
                equal,
                value,
            } => {
                let mut entry = TableIndexEntry::new(
                    self.convert_expression(key)?,
                    self.convert_expression(value)?,
                );
                if self.hold_token_data {
                    let (left, right) = brackets.tokens();
                    entry.set_tokens(TableIndexEntryTokens {
                        opening_bracket: self.convert_token(left),
                        closing_bracket: self.convert_token(right),
                        equal: self.convert_token(equal),
                    })
                }
                entry.into()
            }
            ast::Field::NameKey { key, equal, value } => {
                let mut entry = TableFieldEntry::new(
                    self.convert_token_to_identifier(key),
                    self.convert_expression(value)?,
                );
                if self.hold_token_data {
                    entry.set_token(self.convert_token(equal));
                }
                entry.into()
            }
            ast::Field::NoKey(value) => TableEntry::Value(self.convert_expression(value)?),
            _ => {
                return Err(ConvertError::TableEntry {
                    entry: field.to_string(),
                })
            }
        })
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_binop(&self, operator: &ast::BinOp) -> Result<BinaryOperator, ConvertError> {
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

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_unop(&self, operator: &ast::UnOp) -> Result<UnaryOperator, ConvertError> {
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

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_compound_op(
        &self,
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

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_block(&self, block: &ast::Block) -> Result<Block, ConvertError> {
        let (statements, semicolon_tokens) = if self.hold_token_data {
            let statements_tokens: Result<Vec<_>, ConvertError> = block
                .stmts_with_semicolon()
                .map(|(statement, token)| {
                    self.convert_statement(statement).map(|statement| {
                        (
                            statement,
                            token.as_ref().map(|token| self.convert_token(token)),
                        )
                    })
                })
                .collect();

            let (statements, tokens): (_, Vec<_>) = statements_tokens?.into_iter().unzip();

            (statements, Some(tokens))
        } else {
            let statements: Result<_, _> = block
                .stmts()
                .map(|statement| self.convert_statement(statement))
                .collect();
            (statements?, None)
        };

        let mut new_block = Block::new(
            statements,
            block
                .last_stmt()
                .map(|statement| self.convert_last_statement(statement))
                .transpose()?,
        );

        if let Some(semicolons) = semicolon_tokens {
            let last_semicolon = block.last_stmt_with_semicolon().and_then(|(_, semicolon)| {
                semicolon.as_ref().map(|token| self.convert_token(token))
            });

            new_block.set_tokens(BlockTokens {
                semicolons,
                last_semicolon,
            });
        }

        Ok(new_block)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_ast(&self, ast: Ast) -> Result<Block, ConvertError> {
        self.convert_block(ast.nodes())
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
        let (kind, code) = match self {
            ConvertError::Statement { statement } => ("statement", statement),
            ConvertError::LastStatement { statement } => ("last statement", statement),
            ConvertError::Variable { variable } => ("variable", variable),
            ConvertError::FunctionArguments { arguments } => ("function arguments", arguments),
            ConvertError::Call { call } => ("function call", call),
            ConvertError::Index { index } => ("index expression", index),
            ConvertError::Suffix { suffix } => ("suffix", suffix),
            ConvertError::Prefix { prefix } => ("prefix", prefix),
            ConvertError::Number {
                number,
                parsing_error,
            } => {
                return write!(
                    f,
                    "unable to convert number from `{}` ({})",
                    number, parsing_error
                )
            }
            ConvertError::Expression { expression } => ("expression", expression),
            ConvertError::FunctionParameter { parameter } => ("parameter", parameter),
            ConvertError::FunctionParameters { parameters } => ("parameters", parameters),
            ConvertError::TableEntry { entry } => ("table entry", entry),
            ConvertError::BinaryOperator { operator } => ("binary operator", operator),
            ConvertError::CompoundOperator { operator } => ("compound operator", operator),
            ConvertError::UnaryOperator { operator } => ("unary operator", operator),
        };
        write!(f, "unable to convert {} from `{}`", kind, code)
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
    use crate::nodes::ReturnStatement;

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
                    pretty_assertions::assert_eq!(block, expect_block);
                }
            )*
        };
    }

    test_parse!(
        empty_string("") => Block::default(),
        empty_do("do end") => DoStatement::default(),
        do_return_end("do return end") => DoStatement::new(ReturnStatement::default().into()),
        break_statement("break") => LastStatement::new_break(),
        return_no_values("return") => ReturnStatement::default(),
        return_true("return true") => ReturnStatement::one(Expression::from(true)),
        return_false("return false") => ReturnStatement::one(false),
        return_nil("return nil") => ReturnStatement::one(Expression::nil()),
        return_variable_arguments("return ...") => ReturnStatement::one(Expression::variable_arguments()),
        return_variable("return var") => ReturnStatement::one(Expression::identifier("var")),
        return_parentheses_true("return (true)") => ReturnStatement::one(
            Expression::from(true).in_parentheses(),
        ),
        return_true_false("return true, false") => ReturnStatement::one(Expression::from(true))
            .with_expression(false),
        empty_while_true_do("while true do end") => WhileStatement::new(Block::default(), true),
        while_false_do_break("while false do break end") => WhileStatement::new(
            LastStatement::new_break(),
            false,
        ),
        empty_repeat("repeat until true") => RepeatStatement::new(Block::default(), true),
        repeat_break("repeat break until true") => RepeatStatement::new(
            LastStatement::new_break(),
            true,
        ),
        repeat_continue("repeat continue until true") => RepeatStatement::new(
            LastStatement::new_continue(),
            true,
        ),
        local_assignment_with_no_values("local var") => LocalAssignStatement::from_variable("var"),
        multiple_local_assignment_with_no_values("local foo, bar") => LocalAssignStatement::from_variable("foo")
            .with_variable("bar"),
        local_assignment_with_one_value("local var = true") => LocalAssignStatement::from_variable("var")
            .with_value(true),
        multiple_local_assignment_with_two_values("local foo, bar = true, false") => LocalAssignStatement::from_variable("foo")
            .with_variable("bar")
            .with_value(true)
            .with_value(false),
        return_binary_and("return true and false") => ReturnStatement::one(
            BinaryExpression::new(BinaryOperator::And, true, false),
        ),
        return_zero("return 0") => ReturnStatement::one(
            NumberExpression::from_str("0").unwrap(),
        ),
        return_one("return 1") => ReturnStatement::one(
            NumberExpression::from_str("1").unwrap(),
        ),
        return_float("return 1.5") => ReturnStatement::one(
            NumberExpression::from_str("1.5").unwrap(),
        ),
        return_zero_point_five("return .5") => ReturnStatement::one(
            NumberExpression::from_str(".5").unwrap(),
        ),
        return_not_true("return not true") => ReturnStatement::one(
            UnaryExpression::new(UnaryOperator::Not, true),
        ),
        return_variable_length("return #array") => ReturnStatement::one(
            UnaryExpression::new(
                UnaryOperator::Length,
                Expression::identifier("array"),
            ),
        ),
        return_minus_variable("return -num") => ReturnStatement::one(
            UnaryExpression::new(
                UnaryOperator::Minus,
                Expression::identifier("num"),
            ),
        ),
        call_function("call()") => FunctionCall::from_name("call"),
        call_indexed_table("foo.bar()") => FunctionCall::from_prefix(
            FieldExpression::new(Prefix::from_name("foo"), "bar")
        ),
        call_method("foo:bar()") => FunctionCall::from_name("foo").with_method("bar"),
        call_method_with_one_argument("foo:bar(true)") => FunctionCall::from_name("foo")
            .with_method("bar")
            .with_argument(true),
        call_function_with_one_argument("call(true)") => FunctionCall::from_name("call")
            .with_argument(true),
        call_function_with_two_arguments("call(true, false)") => FunctionCall::from_name("call")
            .with_argument(true)
            .with_argument(false),
        call_chain_empty("call()()") => FunctionCall::from_prefix(
            FunctionCall::from_name("call")
        ),
        call_chain_with_args("call(true)(false)") => FunctionCall::from_prefix(
            FunctionCall::from_name("call").with_argument(true)
        ).with_argument(false),
        call_method_chain_empty("call():method()") => FunctionCall::from_prefix(
            FunctionCall::from_name("call")
        ).with_method("method"),
        call_method_chain_with_arguments("call(true):method(false)") => FunctionCall::from_prefix(
            FunctionCall::from_name("call").with_argument(true)
        ).with_method("method").with_argument(false),
        call_index_chain_empty("call().method()") => FunctionCall::from_prefix(
            FieldExpression::new(FunctionCall::from_name("call"), "method")
        ),
        call_with_empty_table_argument("call{}") => FunctionCall::from_name("call")
            .with_arguments(TableExpression::default()),
        call_with_empty_string_argument("call''") => FunctionCall::from_name("call")
            .with_arguments(StringExpression::empty()),
        return_call_function("return call()") => ReturnStatement::one(
            FunctionCall::from_name("call"),
        ),
        return_call_indexed_table("return foo.bar()") => ReturnStatement::one(
            FunctionCall::from_prefix(FieldExpression::new(Prefix::from_name("foo"), "bar")),
        ),
        return_call_method("return foo:bar()") => ReturnStatement::one(
            FunctionCall::from_name("foo").with_method("bar"),
        ),
        return_call_method_with_one_argument("return foo:bar(true)") => ReturnStatement::one(
            FunctionCall::from_name("foo").with_method("bar").with_argument(true),
        ),
        return_call_function_with_one_argument("return call(true)") => ReturnStatement::one(
            FunctionCall::from_name("call").with_argument(true),
        ),
        return_call_function_with_two_arguments("return call(true, false)") => ReturnStatement::one(
            FunctionCall::from_name("call")
                .with_argument(true)
                .with_argument(false),
        ),
        return_call_chain_empty("return call()()") => ReturnStatement::one(
            FunctionCall::from_prefix(FunctionCall::from_name("call")),
        ),
        return_call_chain_with_args("return call(true)(false)") => ReturnStatement::one(
            FunctionCall::from_prefix(
                FunctionCall::from_name("call").with_argument(true)
            ).with_argument(false),
        ),
        return_call_method_chain_empty("return call():method()") => ReturnStatement::one(
            FunctionCall::from_prefix(FunctionCall::from_name("call")).with_method("method"),
        ),
        return_call_method_chain_with_arguments("return call(true):method(false)")
            => ReturnStatement::one(
                FunctionCall::from_prefix(FunctionCall::from_name("call").with_argument(true))
                    .with_method("method")
                    .with_argument(false),
            ),
        return_call_index_chain_empty("return call().method()") => ReturnStatement::one(
            FunctionCall::from_prefix(FieldExpression::new(FunctionCall::from_name("call"), "method")),
        ),
        return_call_new_empty_function("return (function() end)()") => ReturnStatement::one(
            FunctionCall::from_prefix(
                ParentheseExpression::new(FunctionExpression::default())
            ),
        ),
        return_call_variable_argument("return (...)()") => ReturnStatement::one(
            FunctionCall::from_prefix(ParentheseExpression::new(Expression::variable_arguments())),
        ),
        return_call_variable_in_parentheses("return (var)()") => ReturnStatement::one(
            FunctionCall::from_prefix(ParentheseExpression::new(Expression::identifier("var"))),
        ),
        return_call_variable_in_double_parentheses("return ((var))()") => ReturnStatement::one(
            FunctionCall::from_prefix(
                ParentheseExpression::new(Expression::identifier("var").in_parentheses())
            ),
        ),
        return_field_expression("return math.huge") => ReturnStatement::one(
            FieldExpression::new(Prefix::from_name("math"), "huge")
        ),
        index_field_function_call("return call().result") => ReturnStatement::one(
            FieldExpression::new(FunctionCall::from_name("call"), "result"),
        ),
        return_index_expression("return value[true]") => ReturnStatement::one(
            IndexExpression::new(Prefix::from_name("value"), true)
        ),
        return_empty_table("return {}") => ReturnStatement::one(TableExpression::default()),
        return_array_with_one_element("return {true}") => ReturnStatement::one(
            TableExpression::default().append_array_value(true)
        ),
        return_array_with_two_elements("return {true, false}") => ReturnStatement::one(
            TableExpression::default()
                .append_array_value(true)
                .append_array_value(false)

        ),
        return_array_with_one_field("return { field = true }") => ReturnStatement::one(
            TableExpression::default().append_field("field", true)
        ),
        return_array_with_one_key_expression("return { [false] = true }") => ReturnStatement::one(
            TableExpression::default().append_index(false, true)
        ),
        assign_variable("var = true") => AssignStatement::from_variable(
            Variable::new("var"),
            true,
        ),
        assign_two_variables("var, var2 = true, false") => AssignStatement::from_variable(
            Variable::new("var"),
            true,
        ).append_assignment(Variable::new("var2"), false),
        assign_field("var.field = true") => AssignStatement::from_variable(
            FieldExpression::new(Prefix::from_name("var"), "field"),
            true,
        ),
        assign_index("var[false] = true") => AssignStatement::from_variable(
            IndexExpression::new(Prefix::from_name("var"), false),
            true,
        ),
        return_empty_function("return function() end") => ReturnStatement::one(
            FunctionExpression::default(),
        ),
        return_empty_function_with_one_param("return function(a) end") => ReturnStatement::one(
            FunctionExpression::default().with_parameter("a"),
        ),
        return_empty_function_with_two_params("return function(a, b) end") => ReturnStatement::one(
            FunctionExpression::default().with_parameter("a").with_parameter("b"),
        ),
        return_empty_variadic_function("return function(...) end") => ReturnStatement::one(
            FunctionExpression::default().variadic(),
        ),
        return_empty_variadic_function_with_one_param("return function(a, ...) end")
            => ReturnStatement::one(
                FunctionExpression::default().with_parameter("a").variadic(),
            ),
        return_function_that_returns("return function() return true end")
            => ReturnStatement::one(
                FunctionExpression::from_block(ReturnStatement::one(Expression::from(true)))
            ),
        empty_if_statement("if true then end") => IfStatement::create(true, Block::default()),
        if_statement_returns("if true then return end") => IfStatement::create(
            Expression::from(true),
            ReturnStatement::default(),
        ),
        empty_if_statement_with_empty_else("if true then else end")
            => IfStatement::create(true, Block::default())
                .with_else_block(Block::default()),
        empty_if_statement_with_empty_elseif("if true then elseif false then end")
            => IfStatement::create(true, Block::default())
                .with_new_branch(false, Block::default()),
        empty_if_statement_with_empty_elseif_and_empty_else("if true then elseif false then else end")
            => IfStatement::create(true, Block::default())
                .with_new_branch(false, Block::default())
                .with_else_block(Block::default()),
        empty_if_statement_with_returning_else("if true then else return end")
            => IfStatement::create(true, Block::default())
                .with_else_block(ReturnStatement::default()),
        empty_local_function("local function name() end")
            => LocalFunctionStatement::from_name("name", Block::default()),
        empty_local_function_variadic("local function name(...) end")
            => LocalFunctionStatement::from_name("name", Block::default()).variadic(),
        empty_local_function_variadic_with_one_parameter("local function name(a, ...) end")
            => LocalFunctionStatement::from_name("name", Block::default())
                .with_parameter("a")
                .variadic(),
        local_function_return("local function name() return end")
            => LocalFunctionStatement::from_name("name", ReturnStatement::default()),

        empty_function_statement("function name() end")
            => FunctionStatement::from_name("name", Block::default()),
        empty_function_statement_variadic("function name(...) end")
            => FunctionStatement::from_name("name", Block::default()).variadic(),
        empty_function_statement_variadic_with_one_parameter("function name(a, ...) end")
            => FunctionStatement::from_name("name", Block::default())
                .with_parameter("a")
                .variadic(),
        function_statement_return("function name() return end")
            => FunctionStatement::from_name("name", ReturnStatement::default()),
        empty_generic_for("for key in pairs(t) do end") => GenericForStatement::new(
            vec!["key".into()],
            vec![
                FunctionCall::from_name("pairs")
                    .with_argument(Expression::identifier("t"))
                    .into(),
            ],
            Block::default(),
        ),
        empty_generic_for_multiple_variables("for key, value in pairs(t) do end") => GenericForStatement::new(
            vec!["key".into(), "value".into()],
            vec![
                FunctionCall::from_name("pairs")
                    .with_argument(Expression::identifier("t"))
                    .into(),
            ],
            Block::default(),
        ),
        empty_generic_for_multiple_values("for key in next, t do end") => GenericForStatement::new(
            vec!["key".into()],
            vec![Expression::identifier("next"), Expression::identifier("t")],
            Block::default(),
        ),
        generic_for_break("for key in pairs(t) do break end") => GenericForStatement::new(
            vec!["key".into()],
            vec![
                FunctionCall::from_name("pairs")
                    .with_argument(Expression::identifier("t"))
                    .into(),
            ],
            LastStatement::new_break(),
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
            LastStatement::new_break(),
        ),
        compound_increment("var += amount") => CompoundAssignStatement::new(
            CompoundOperator::Plus,
            Variable::new("var"),
            Expression::identifier("amount"),
        ),
    );

    mod parse_with_tokens {
        use super::*;

        macro_rules! test_parse_block_with_tokens {
            ($($name:ident($input:literal) => $value:expr),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        let parser = Parser::default().preserve_tokens();
                        let block = parser.parse($input)
                            .expect(&format!("failed to parse `{}`", $input));

                        let expect_block = $value;

                        pretty_assertions::assert_eq!(block, expect_block);
                    }
                )*
            };
        }

        macro_rules! test_parse_statement_with_tokens {
            ($($name:ident($input:literal) => $value:expr),* $(,)?) => {
                test_parse_block_with_tokens!(
                    $(
                        $name($input) => Block::from($value).with_tokens(BlockTokens {
                            semicolons: vec![None],
                            last_semicolon: None,
                        }),
                    )*
                );
            };
        }

        macro_rules! test_parse_last_statement_with_tokens {
            ($($name:ident($input:literal) => $value:expr),* $(,)?) => {
                test_parse_block_with_tokens!(
                    $(
                        $name($input) => Block::from($value).with_tokens(BlockTokens {
                            semicolons: Vec::new(),
                            last_semicolon: None,
                        }),
                    )*
                );
            };
        }

        fn create_true(start: usize, whitespace_length: usize) -> Expression {
            let end = start + 4;
            let token = Token::new_with_line(start, end, 1);
            Expression::True(Some(if whitespace_length == 0 {
                token
            } else {
                token.with_trailing_trivia(TriviaKind::Whitespace.at(
                    end,
                    end + whitespace_length,
                    1,
                ))
            }))
        }

        fn create_identifier(
            identifier: &str,
            start: usize,
            whitespace_length: usize,
        ) -> Identifier {
            create_identifier_at_line(identifier, start, whitespace_length, 1)
        }

        fn create_identifier_at_line(
            identifier: &str,
            start: usize,
            whitespace_length: usize,
            line: usize,
        ) -> Identifier {
            let end = start + identifier.len();
            Identifier::new(identifier).with_token({
                let token = Token::new_with_line(start, end, line);
                if whitespace_length != 0 {
                    token.with_trailing_trivia(TriviaKind::Whitespace.at(
                        end,
                        end + whitespace_length,
                        line,
                    ))
                } else {
                    token
                }
            })
        }

        fn spaced_token(start: usize, end: usize) -> Token {
            spaced_token_at_line(start, end, 1)
        }

        fn spaced_token_at_line(start: usize, end: usize, line: usize) -> Token {
            Token::new_with_line(start, end, line).with_trailing_trivia(TriviaKind::Whitespace.at(
                end,
                end + 1,
                line,
            ))
        }

        fn default_block() -> Block {
            Block::default().with_tokens(BlockTokens {
                semicolons: Vec::new(),
                last_semicolon: None,
            })
        }

        fn token_at_first_line(start: usize, end: usize) -> Token {
            Token::new_with_line(start, end, 1)
        }

        test_parse_last_statement_with_tokens!(
            return_with_comment("return -- comment") => ReturnStatement::default()
                .with_tokens(ReturnTokens {
                    r#return: token_at_first_line(0, 6)
                        .with_trailing_trivia(TriviaKind::Whitespace.at(6, 7, 1))
                        .with_trailing_trivia(TriviaKind::Comment.at(7, 17, 1)),
                    commas: Vec::new(),
                }),
            return_true("return true") => ReturnStatement::one(create_true(7, 0))
                .with_tokens(ReturnTokens {
                    r#return: spaced_token(0, 6),
                    commas: Vec::new(),
                }),
            return_false("return false") => ReturnStatement::one(
                Expression::False(Some(token_at_first_line(7, 12)))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_nil("return nil") => ReturnStatement::one(
                Expression::Nil(Some(token_at_first_line(7, 10)))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_variable_arguments("return ...") => ReturnStatement::one(
                Expression::VariableArguments(Some(token_at_first_line(7, 10)))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_empty_single_quote_string("return ''") => ReturnStatement::one(
                StringExpression::empty().with_token(token_at_first_line(7, 9))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_empty_double_quote_string("return \"\"") => ReturnStatement::one(
                StringExpression::empty().with_token(token_at_first_line(7, 9))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_double_quote_string("return \"abc\"") => ReturnStatement::one(
                StringExpression::from_value("abc").with_token(token_at_first_line(7, 12))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_integer_number("return 123") => ReturnStatement::one(
                DecimalNumber::new(123.0).with_token(token_at_first_line(7, 10))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_float("return 12.34 -- value") => ReturnStatement::one(
                DecimalNumber::new(12.34).with_token(
                    spaced_token(7, 12).with_trailing_trivia(TriviaKind::Comment.at(13, 21, 1))
                )
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_binary_number("return 0b1010") => ReturnStatement::one(
                BinaryNumber::new(0b1010, false).with_token(token_at_first_line(7, 13))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_hexadecimal_number("return 0x12EF") => ReturnStatement::one(
                HexNumber::new(0x12EF, false).with_token(token_at_first_line(7, 13))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_empty_table("return {--[[ inside ]]}") => ReturnStatement::one(
                TableExpression::default().with_tokens(TableTokens {
                    opening_brace: token_at_first_line(7, 8)
                        .with_trailing_trivia(TriviaKind::Comment.at(8, 22, 1)),
                    closing_brace: token_at_first_line(22, 23),
                    separators: Vec::new(),
                })
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_array_with_one_element("return { true} ") => ReturnStatement::one(
                TableExpression::default()
                    .append_array_value(create_true(9, 0))
                    .with_tokens(TableTokens {
                        opening_brace: spaced_token(7, 8),
                        closing_brace: spaced_token(13, 14),
                        separators: Vec::new(),
                    })
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_array_with_two_elements("return {true, true}") => ReturnStatement::one(
                TableExpression::default()
                    .append_array_value(create_true(8, 0))
                    .append_array_value(create_true(14, 0))
                    .with_tokens(TableTokens {
                        opening_brace: token_at_first_line(7, 8),
                        closing_brace: token_at_first_line(18, 19),
                        separators: vec![spaced_token(12, 13)],
                    })
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_array_with_one_field("return { field = true; }") => ReturnStatement::one(
                TableExpression::default()
                    .append_entry(
                        TableFieldEntry::new(
                            create_identifier("field", 9, 1),
                            create_true(17, 0),
                        ).with_token(spaced_token(15, 16))
                    )
                    .with_tokens(TableTokens {
                        opening_brace: spaced_token(7, 8),
                        closing_brace: token_at_first_line(23, 24),
                        separators: vec![spaced_token(21, 22)],
                    })
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_array_with_one_key_expression("return { [var] = true }") => ReturnStatement::one(
                TableExpression::default()
                    .append_entry(
                        TableIndexEntry::new(
                            create_identifier("var", 10, 0),
                            create_true(17, 1),
                        ).with_tokens(TableIndexEntryTokens {
                            opening_bracket: token_at_first_line(9, 10),
                            closing_bracket: spaced_token(13, 14),
                            equal: spaced_token(15, 16),
                        })
                    )
                    .with_tokens(TableTokens {
                        opening_brace: spaced_token(7, 8),
                        closing_brace: token_at_first_line(22, 23),
                        separators: Vec::new(),
                    })
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_field_expression("return math.huge") => ReturnStatement::one(
                FieldExpression::new(
                    Prefix::from_name(create_identifier("math", 7, 0)),
                    create_identifier("huge", 12, 0)
                ).with_token(token_at_first_line(11, 12))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_double_field_expression("return table.ok .result") => ReturnStatement::one(
                FieldExpression::new(
                    FieldExpression::new(
                        Prefix::from_name(create_identifier("table", 7, 0)),
                        create_identifier("ok", 13, 1)
                    ).with_token(token_at_first_line(12, 13)),
                    create_identifier("result", 17, 0)
                ).with_token(token_at_first_line(16, 17))
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_index_expression("return value [ true ] ") => ReturnStatement::one(
                IndexExpression::new(
                    create_identifier("value", 7, 1),
                    create_true(15, 1)
                ).with_tokens(IndexExpressionTokens {
                    opening_bracket: spaced_token(13, 14),
                    closing_bracket: spaced_token(20, 21),
                })
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_true_and_true("return true and true") => ReturnStatement::default()
                .with_expression(
                    BinaryExpression::new(
                        BinaryOperator::And,
                        create_true(7, 1),
                        create_true(16, 0),
                    ).with_token(spaced_token(12, 15))
                )
                .with_tokens(ReturnTokens {
                    r#return: spaced_token(0, 6),
                    commas: Vec::new(),
                }),
            return_not_true("return not true") => ReturnStatement::default()
                .with_expression(
                    UnaryExpression::new(
                        UnaryOperator::Not,
                        create_true(11, 0),
                    ).with_token(spaced_token(7, 10))
                )
                .with_tokens(ReturnTokens {
                    r#return: spaced_token(0, 6),
                    commas: Vec::new(),
                }),
            return_parenthese_expression("return ( true )") => ReturnStatement::default()
                .with_expression(
                    ParentheseExpression::new(create_true(9, 1))
                        .with_tokens(
                            ParentheseTokens {
                                left_parenthese: spaced_token(7, 8),
                                right_parenthese: token_at_first_line(14, 15),
                            }
                        )
                )
                .with_tokens(ReturnTokens {
                    r#return: spaced_token(0, 6),
                    commas: Vec::new(),
                }),
            return_empty_function("return function  ( --[[params]]) end") => ReturnStatement::one(
                FunctionExpression::from_block(default_block())
                    .with_tokens(FunctionExpressionTokens {
                        function: token_at_first_line(7, 15)
                            .with_trailing_trivia(TriviaKind::Whitespace.at(15, 17, 1)),
                        opening_parenthese: token_at_first_line(17, 18)
                            .with_trailing_trivia(TriviaKind::Whitespace.at(18, 19, 1))
                            .with_trailing_trivia(TriviaKind::Comment.at(19, 31, 1)),
                        closing_parenthese: spaced_token(31, 32),
                        end: token_at_first_line(33, 36),
                        parameter_commas: Vec::new(),
                        variable_arguments: None,
                    }),
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_empty_function_with_one_param("return function(a )end") => ReturnStatement::one(
                FunctionExpression::from_block(default_block()).with_parameter(
                    create_identifier("a", 16, 1)
                ).with_tokens(FunctionExpressionTokens {
                    function: token_at_first_line(7, 15),
                    opening_parenthese: token_at_first_line(15, 16),
                    closing_parenthese: token_at_first_line(18, 19),
                    end: token_at_first_line(19, 22),
                    parameter_commas: Vec::new(),
                    variable_arguments: None,
                }),
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_empty_function_with_two_params("return function(a, b--[[foo]]) end") => ReturnStatement::one(
                FunctionExpression::from_block(default_block())
                    .with_parameter(Identifier::new("a").with_token(token_at_first_line(16, 17)))
                    .with_parameter(
                        Identifier::new("b").with_token(
                            token_at_first_line(19, 20).with_trailing_trivia(TriviaKind::Comment.at(20, 29, 1))
                        )
                    )
                    .with_tokens(FunctionExpressionTokens {
                        function: token_at_first_line(7, 15),
                        opening_parenthese: token_at_first_line(15, 16),
                        closing_parenthese: spaced_token(29, 30),
                        end: token_at_first_line(31, 34),
                        parameter_commas: vec![spaced_token(17, 18)],
                        variable_arguments: None,
                    }),
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_empty_variadic_function("return function(... ) end") => ReturnStatement::one(
                FunctionExpression::from_block(default_block())
                    .variadic()
                    .with_tokens(FunctionExpressionTokens {
                        function: token_at_first_line(7, 15),
                        opening_parenthese: token_at_first_line(15, 16),
                        closing_parenthese: spaced_token(20, 21),
                        end: token_at_first_line(22, 25),
                        parameter_commas: Vec::new(),
                        variable_arguments: Some(spaced_token(16, 19)),
                    }),
            ).with_tokens(ReturnTokens {
                r#return: spaced_token(0, 6),
                commas: Vec::new(),
            }),
            return_two_values("return true ,  true--end") => ReturnStatement::default()
                .with_expression(create_true(7, 1))
                .with_expression(Expression::True(Some(
                    token_at_first_line(15, 19).with_trailing_trivia(TriviaKind::Comment.at(19, 24, 1))
                )))
                .with_tokens(ReturnTokens {
                    r#return: spaced_token(0, 6),
                    commas: vec![
                        token_at_first_line(12, 13).with_trailing_trivia(TriviaKind::Whitespace.at(13, 15, 1))
                    ],
                }),
            return_variable("return var") => ReturnStatement::default()
                .with_expression(
                    Identifier::new("var").with_token(token_at_first_line(7, 10))
                )
                .with_tokens(ReturnTokens {
                    r#return: spaced_token(0, 6),
                    commas: Vec::new(),
                }),
            break_statement("break") => LastStatement::Break(Some(token_at_first_line(0, 5))),
            break_statement_with_comment("break-- bye") => LastStatement::Break(Some(
                token_at_first_line(0, 5).with_trailing_trivia(TriviaKind::Comment.at(5, 11, 1))
            )),
            continue_statement("continue") => LastStatement::Continue(Some(token_at_first_line(0, 8))),
            continue_statement_with_comment("continue-- bye") => LastStatement::Continue(Some(
                token_at_first_line(0, 8).with_trailing_trivia(TriviaKind::Comment.at(8, 14, 1))
            )),
        );

        test_parse_statement_with_tokens!(
            empty_local_function("local function name ()end") => LocalFunctionStatement::from_name(
                create_identifier("name", 15, 1),
                default_block()
            ).with_tokens(LocalFunctionTokens {
                local: spaced_token(0, 5),
                function: spaced_token(6, 14),
                opening_parenthese: token_at_first_line(20, 21),
                closing_parenthese: token_at_first_line(21, 22),
                end: token_at_first_line(22, 25),
                parameter_commas: Vec::new(),
                variable_arguments: None,
            }),
            empty_local_function_variadic("local function name(...)end") => LocalFunctionStatement::from_name(
                Identifier::new("name").with_token(token_at_first_line(15, 19)),
                default_block(),
            )
            .variadic()
            .with_tokens(LocalFunctionTokens {
                local: spaced_token(0, 5),
                function: spaced_token(6, 14),
                opening_parenthese: token_at_first_line(19, 20),
                closing_parenthese: token_at_first_line(23, 24),
                end: token_at_first_line(24, 27),
                parameter_commas: Vec::new(),
                variable_arguments: Some(token_at_first_line(20, 23)),
            }),
            empty_local_function_variadic_with_one_parameter("local function name(a,b) end")
                => LocalFunctionStatement::from_name(
                    Identifier::new("name").with_token(token_at_first_line(15, 19)),
                    default_block(),
                )
                .with_parameter(Identifier::new("a").with_token(token_at_first_line(20, 21)))
                .with_parameter(Identifier::new("b").with_token(token_at_first_line(22, 23)))
                .with_tokens(LocalFunctionTokens {
                    local: spaced_token(0, 5),
                    function: spaced_token(6, 14),
                    opening_parenthese: token_at_first_line(19, 20),
                    closing_parenthese: spaced_token(23, 24),
                    end: token_at_first_line(25, 28),
                    parameter_commas: vec![token_at_first_line(21, 22)],
                    variable_arguments: None,
                }),
            call_function("call()") => FunctionCall::from_name(
                create_identifier("call", 0, 0)
            ).with_arguments(TupleArguments::default().with_tokens(TupleArgumentsTokens {
                opening_parenthese: token_at_first_line(4, 5),
                closing_parenthese: token_at_first_line(5, 6),
                commas: Vec::new(),
            })).with_tokens(FunctionCallTokens {
                colon: None,
            }),
            call_indexed_table("foo.bar()") => FunctionCall::from_prefix(
                FieldExpression::new(
                    create_identifier("foo", 0, 0),
                    create_identifier("bar", 4, 0)
                ).with_token(token_at_first_line(3, 4))
            ).with_arguments(TupleArguments::default().with_tokens(TupleArgumentsTokens {
                opening_parenthese: token_at_first_line(7, 8),
                closing_parenthese: token_at_first_line(8, 9),
                commas: Vec::new(),
            })).with_tokens(FunctionCallTokens {
                colon: None,
            }),
            call_method("foo: bar()") => FunctionCall::from_name(create_identifier("foo", 0, 0))
                .with_method(create_identifier("bar", 5, 0))
                .with_arguments(TupleArguments::default().with_tokens(TupleArgumentsTokens {
                    opening_parenthese: token_at_first_line(8, 9),
                    closing_parenthese: token_at_first_line(9, 10),
                    commas: Vec::new(),
                }))
                .with_tokens(FunctionCallTokens {
                    colon: Some(spaced_token(3, 4)),
                }),
            call_method_with_one_argument("foo:bar( true )") => FunctionCall::from_name(
                create_identifier("foo", 0, 0)
            )
            .with_method(create_identifier("bar", 4, 0))
            .with_arguments(
                TupleArguments::default()
                    .with_argument(create_true(9, 1))
                    .with_tokens(TupleArgumentsTokens {
                        opening_parenthese: spaced_token(7, 8),
                        closing_parenthese: token_at_first_line(14, 15),
                        commas: Vec::new(),
                    })
                )
            .with_tokens(FunctionCallTokens {
                colon: Some(token_at_first_line(3, 4)),
            }),
            call_function_with_one_argument("call ( true ) ") =>  FunctionCall::from_name(
                create_identifier("call", 0, 1)
            )
            .with_arguments(
                TupleArguments::default()
                    .with_argument(create_true(7, 1))
                    .with_tokens(TupleArgumentsTokens {
                        opening_parenthese: spaced_token(5, 6),
                        closing_parenthese: spaced_token(12, 13),
                        commas: Vec::new(),
                    })
                )
            .with_tokens(FunctionCallTokens {
                colon: None,
            }),
            call_function_with_two_arguments("call(true, true)") =>  FunctionCall::from_name(
                create_identifier("call", 0, 0)
            )
            .with_arguments(
                TupleArguments::default()
                    .with_argument(create_true(5, 0))
                    .with_argument(create_true(11, 0))
                    .with_tokens(TupleArgumentsTokens {
                        opening_parenthese: token_at_first_line(4, 5),
                        closing_parenthese: token_at_first_line(15, 16),
                        commas: vec![spaced_token(9, 10)],
                    })
                )
            .with_tokens(FunctionCallTokens {
                colon: None,
            }),
            call_chain_with_args("call(true)( )") => FunctionCall::from_prefix(
                FunctionCall::from_name(create_identifier("call", 0, 0))
                    .with_arguments(
                        TupleArguments::default()
                            .with_argument(create_true(5, 0))
                            .with_tokens(TupleArgumentsTokens {
                                opening_parenthese: token_at_first_line(4, 5),
                                closing_parenthese: token_at_first_line(9, 10),
                                commas: Vec::new(),
                            })
                        )
                    .with_tokens(FunctionCallTokens {
                        colon: None,
                    }),
            )
            .with_arguments(TupleArguments::default().with_tokens(TupleArgumentsTokens {
                opening_parenthese: spaced_token(10, 11),
                closing_parenthese: token_at_first_line(12, 13),
                commas: Vec::new(),
            }))
            .with_tokens(FunctionCallTokens {
                colon: None,
            }),
            call_with_empty_table_argument("call{ }") => FunctionCall::from_name(
                create_identifier("call", 0, 0)
            ).with_arguments(TableExpression::default().with_tokens(TableTokens {
                opening_brace: spaced_token(4, 5),
                closing_brace: token_at_first_line(6, 7),
                separators: Vec::new(),
            })).with_tokens(FunctionCallTokens {
                colon: None,
            }),
            call_with_empty_string_argument("call ''") => FunctionCall::from_name(
                create_identifier("call", 0, 1)
            ).with_arguments(
                StringExpression::empty().with_token(token_at_first_line(5, 7))
            ).with_tokens(FunctionCallTokens {
                colon: None,
            }),
            empty_do("do end") => DoStatement::new(default_block())
                .with_tokens(DoTokens {
                    r#do: spaced_token(0, 2),
                    end: token_at_first_line(3, 6),
                }),
            empty_do_with_long_comment("do --[[ hello ]] end") => DoStatement::new(default_block())
                .with_tokens(DoTokens {
                    r#do: token_at_first_line(0, 2)
                        .with_trailing_trivia(TriviaKind::Whitespace.at(2, 3, 1))
                        .with_trailing_trivia(TriviaKind::Comment.at(3, 16, 1))
                        .with_trailing_trivia(TriviaKind::Whitespace.at(16, 17, 1)),
                    end: token_at_first_line(17, 20),
                }),
            assign_variable("var = true") => AssignStatement::from_variable(
                create_identifier("var", 0, 1),
                Expression::True(Some(token_at_first_line(6, 10))),
            ).with_tokens(AssignTokens {
                equal: spaced_token(4, 5),
                variable_commas: Vec::new(),
                value_commas: Vec::new(),
            }),
            assign_two_variables_with_two_values("var, var2 = true, true") => AssignStatement::from_variable(
                Identifier::new("var").with_token(token_at_first_line(0, 3)),
                create_true(12, 0),
            ).append_assignment(
                create_identifier("var2", 5, 1),
                Expression::True(Some(token_at_first_line(18, 22))),
            ).with_tokens(AssignTokens {
                equal: spaced_token(10, 11),
                variable_commas: vec![spaced_token(3, 4)],
                value_commas: vec![spaced_token(16, 17)],
            }),
            empty_function_statement("function name() end")
                => FunctionStatement::new(
                    FunctionName::from_name(
                        Identifier::new("name").with_token(token_at_first_line(9, 13))
                    ).with_tokens(FunctionNameTokens { periods: Vec::new(), colon: None }),
                    default_block(),
                    Vec::new(),
                    false,
                ).with_tokens(FunctionStatementTokens {
                    function: spaced_token(0, 8),
                    opening_parenthese: token_at_first_line(13, 14),
                    closing_parenthese: spaced_token(14, 15),
                    end: token_at_first_line(16, 19),
                    parameter_commas: Vec::new(),
                    variable_arguments: None
                }),
            empty_function_statement_with_field("function name.field ()end")
                => FunctionStatement::new(
                    FunctionName::from_name(
                        Identifier::new("name").with_token(token_at_first_line(9, 13))
                    ).with_field(
                        Identifier::new("field").with_token(spaced_token(14, 19))
                    ).with_tokens(FunctionNameTokens {
                        periods: vec![token_at_first_line(13, 14)],
                        colon: None,
                    }),
                    default_block(),
                    Vec::new(),
                    false,
                ).with_tokens(FunctionStatementTokens {
                    function: spaced_token(0, 8),
                    opening_parenthese: token_at_first_line(20, 21),
                    closing_parenthese: token_at_first_line(21, 22),
                    end: token_at_first_line(22, 25),
                    parameter_commas: Vec::new(),
                    variable_arguments: None
                }),
            empty_function_statement_with_method("function name:method ()end")
                => FunctionStatement::new(
                    FunctionName::from_name(
                        Identifier::new("name").with_token(token_at_first_line(9, 13))
                    )
                    .with_method(create_identifier("method", 14, 1))
                    .with_tokens(FunctionNameTokens {
                        periods: Vec::new(),
                        colon: Some(token_at_first_line(13, 14)),
                    }),
                    default_block(),
                    Vec::new(),
                    false,
                ).with_tokens(FunctionStatementTokens {
                    function: spaced_token(0, 8),
                    opening_parenthese: token_at_first_line(21, 22),
                    closing_parenthese: token_at_first_line(22, 23),
                    end: token_at_first_line(23, 26),
                    parameter_commas: Vec::new(),
                    variable_arguments: None
                }),
            empty_function_statement_variadic("function name(...) end")
                => FunctionStatement::new(
                    FunctionName::from_name(
                        Identifier::new("name").with_token(token_at_first_line(9, 13))
                    ).with_tokens(FunctionNameTokens { periods: Vec::new(), colon: None }),
                    default_block(),
                    Vec::new(),
                    true,
                ).with_tokens(FunctionStatementTokens {
                    function: spaced_token(0, 8),
                    opening_parenthese: token_at_first_line(13, 14),
                    closing_parenthese: token_at_first_line(17, 18)
                        .with_trailing_trivia(TriviaKind::Whitespace.at(18, 19, 1)),
                    end: token_at_first_line(19, 22),
                    parameter_commas: Vec::new(),
                    variable_arguments: Some(token_at_first_line(14, 17))
                }),
            empty_function_statement_variadic_with_one_parameter("function name(a,...)end")
                => FunctionStatement::new(
                    FunctionName::from_name(
                        Identifier::new("name").with_token(token_at_first_line(9, 13))
                    ).with_tokens(FunctionNameTokens { periods: Vec::new(), colon: None }),
                    default_block(),
                    vec![
                        Identifier::new("a").with_token(token_at_first_line(14, 15))
                    ],
                    true,
                ).with_tokens(FunctionStatementTokens {
                    function: spaced_token(0, 8),
                    opening_parenthese: token_at_first_line(13, 14),
                    closing_parenthese: token_at_first_line(19, 20),
                    end: token_at_first_line(20, 23),
                    parameter_commas: vec![
                        token_at_first_line(15, 16),
                    ],
                    variable_arguments: Some(token_at_first_line(16, 19))
                }),
            empty_generic_for("for key in foo do end") => GenericForStatement::new(
                vec![
                    create_identifier("key", 4, 1),
                ],
                vec![
                    create_identifier("foo", 11, 1).into(),
                ],
                default_block(),
            ).with_tokens(GenericForTokens {
                r#for: spaced_token(0, 3),
                r#in: spaced_token(8, 10),
                r#do: spaced_token(15, 17),
                end: token_at_first_line(18, 21),
                identifier_commas: Vec::new(),
                value_commas: Vec::new(),
            }),
            empty_generic_for_multiple_variables("for key, value in foo do end") => GenericForStatement::new(
                vec![
                    Identifier::new("key").with_token(token_at_first_line(4, 7)),
                    create_identifier("value", 9, 1),
                ],
                vec![
                    create_identifier("foo", 18, 1).into(),
                ],
                default_block(),
            ).with_tokens(GenericForTokens {
                r#for: spaced_token(0, 3),
                r#in: spaced_token(15, 17),
                r#do: spaced_token(22, 24),
                end: token_at_first_line(25, 28),
                identifier_commas: vec![spaced_token(7, 8)],
                value_commas: Vec::new(),
            }),
            empty_generic_for_multiple_values("for key in next , t do end") => GenericForStatement::new(
                vec![create_identifier("key", 4, 1)],
                vec![
                    create_identifier("next", 11, 1).into(),
                    create_identifier("t", 18, 1).into(),
                ],
                default_block(),
            ).with_tokens(GenericForTokens {
                r#for: spaced_token(0, 3),
                r#in: spaced_token(8, 10),
                r#do: spaced_token(20, 22),
                end: token_at_first_line(23, 26),
                identifier_commas: Vec::new(),
                value_commas: vec![
                    token_at_first_line(16, 17).with_trailing_trivia(TriviaKind::Whitespace.at(17, 18, 1)),
                ],
            }),
            empty_if_statement("if true then end") => IfStatement::create(
                create_true(3, 1),
                default_block()
            ).with_tokens(IfStatementTokens {
                r#if: token_at_first_line(0, 2).with_trailing_trivia(TriviaKind::Whitespace.at(2, 3, 1)),
                then: token_at_first_line(8, 12).with_trailing_trivia(TriviaKind::Whitespace.at(12, 13, 1)),
                end: token_at_first_line(13, 16),
                r#else: None,
            }),
            empty_if_statement_with_empty_else("if true then else end") => IfStatement::create(
                create_true(3, 1),
                default_block()
            )
            .with_else_block(default_block())
            .with_tokens(IfStatementTokens {
                r#if: spaced_token(0, 2),
                then: spaced_token(8, 12),
                end: token_at_first_line(18, 21),
                r#else: Some(spaced_token(13, 17)),
            }),
            empty_if_statement_with_empty_elseif("if true then elseif true then end")
                => IfStatement::create(create_true(3, 1), default_block())
                .with_branch(
                    IfBranch::new(create_true(20, 1), default_block())
                        .with_tokens(IfBranchTokens {
                            elseif: spaced_token(13, 19),
                            then: spaced_token(25, 29),
                        })
                )
                .with_tokens(IfStatementTokens {
                    r#if: spaced_token(0, 2),
                    then: spaced_token(8, 12),
                    end: token_at_first_line(30, 33),
                    r#else: None,
                }),
            local_assignment_with_no_values("local var ") => LocalAssignStatement::from_variable(
                create_identifier("var", 6, 1),
            ).with_tokens(LocalAssignTokens {
                local: spaced_token(0, 5),
                equal: None,
                variable_commas: Vec::new(),
                value_commas: Vec::new(),
             }),
            multiple_local_assignment_with_no_values("local foo, bar") => LocalAssignStatement::from_variable(
                create_identifier("foo", 6, 0)
            )
            .with_variable(create_identifier("bar", 11, 0))
            .with_tokens(LocalAssignTokens {
                local: spaced_token(0, 5),
                equal: None,
                variable_commas: vec![spaced_token(9, 10)],
                value_commas: Vec::new(),
             }),
            multiple_local_assignment_with_two_values("local foo, bar = true, true")
                => LocalAssignStatement::from_variable(
                    create_identifier("foo", 6, 0)
                )
                .with_variable(create_identifier("bar", 11, 1))
                .with_value(create_true(17, 0))
                .with_value(create_true(23, 0))
                .with_tokens(LocalAssignTokens {
                    local: spaced_token(0, 5),
                    equal: Some(spaced_token(15, 16)),
                    variable_commas: vec![spaced_token(9, 10)],
                    value_commas: vec![spaced_token(21, 22)],
                 }),

            empty_numeric_for("for i = start,bound do end") => NumericForStatement::new(
                create_identifier("i", 4, 1),
                create_identifier("start", 8, 0),
                create_identifier("bound", 14, 1),
                None,
                default_block(),
            ).with_tokens(NumericForTokens {
                r#for: spaced_token(0, 3),
                equal: spaced_token(6, 7),
                r#do: spaced_token(20, 22),
                end: token_at_first_line(23, 26),
                end_comma: token_at_first_line(13, 14),
                step_comma: None,
            }),
            empty_numeric_for_with_step("for i = start , bound , step do end")
                => NumericForStatement::new(
                    create_identifier("i", 4, 1),
                    create_identifier("start", 8, 1),
                    create_identifier("bound", 16, 1),
                    Some(create_identifier("step", 24, 1).into()),
                    default_block(),
                ).with_tokens(NumericForTokens {
                    r#for: spaced_token(0, 3),
                    equal: spaced_token(6, 7),
                    r#do: spaced_token(29, 31),
                    end: token_at_first_line(32, 35),
                    end_comma: spaced_token(14, 15),
                    step_comma: Some(spaced_token(22, 23)),
                }),
            empty_repeat("repeat until true") => RepeatStatement::new(
                default_block(),
                create_true(13, 0),
            ).with_tokens(RepeatTokens {
                repeat: spaced_token(0, 6),
                until: spaced_token(7, 12),
            }),
            empty_while("while true do end") => WhileStatement::new(
                default_block(),
                create_true(6, 1),
            ).with_tokens(WhileTokens {
                r#while: token_at_first_line(0, 5)
                    .with_trailing_trivia(TriviaKind::Whitespace.at(5, 6, 1)),
                r#do: token_at_first_line(11, 13)
                    .with_trailing_trivia(TriviaKind::Whitespace.at(13, 14, 1)),
                end: token_at_first_line(14, 17),
            }),
            compound_increment("var += amount") => CompoundAssignStatement::new(
                CompoundOperator::Plus,
                create_identifier("var", 0, 1),
                create_identifier("amount", 7, 0),
            ).with_tokens(CompoundAssignTokens { operator: spaced_token(4, 6) }),
        );

        test_parse_block_with_tokens!(
            return_nothing_with_semicolon("return;") => Block::from(
                ReturnStatement::default()
                    .with_tokens(ReturnTokens {
                        r#return: token_at_first_line(0, 6),
                        commas: Vec::new(),
                    }),
            ).with_tokens(BlockTokens {
                semicolons: vec![],
                last_semicolon: Some(token_at_first_line(6, 7)),
            }),
            return_nothing_with_semicolon_and_comment("return; -- return nothing") => Block::from(
                ReturnStatement::default()
                    .with_tokens(ReturnTokens {
                        r#return: token_at_first_line(0, 6),
                        commas: Vec::new(),
                    }),
            ).with_tokens(BlockTokens {
                semicolons: vec![],
                last_semicolon: Some(
                    token_at_first_line(6, 7)
                        .with_trailing_trivia(TriviaKind::Whitespace.at(7, 8, 1))
                        .with_trailing_trivia(TriviaKind::Comment.at(8, 25, 1))
                ),
            }),
            two_local_declarations("local a;\nlocal b;\n") => Block::from(
                LocalAssignStatement::from_variable(create_identifier("a", 6, 0))
                    .with_tokens(LocalAssignTokens {
                        local: spaced_token(0, 5),
                        equal: None,
                        variable_commas: Vec::new(),
                        value_commas: Vec::new(),
                    })
            ).with_statement(
                LocalAssignStatement::from_variable(create_identifier_at_line("b", 15, 0, 2))
                    .with_tokens(LocalAssignTokens {
                        local: spaced_token_at_line(9, 14, 2),
                        equal: None,
                        variable_commas: Vec::new(),
                        value_commas: Vec::new(),
                    })
            ).with_tokens(BlockTokens {
                semicolons: vec![
                    Some(spaced_token(7, 8)),
                    Some(spaced_token_at_line(16, 17, 2)),
                ],
                last_semicolon: None,
            }),
        );
    }
}
