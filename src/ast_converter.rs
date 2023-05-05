use std::{fmt, str::FromStr};

use full_moon::{
    ast,
    tokenizer::{self, InterpolatedStringKind, Symbol, TokenType},
};

use crate::nodes::*;

#[derive(Debug, Default)]
pub(crate) struct AstConverter<'a> {
    hold_token_data: bool,
    work_stack: Vec<ConvertWork<'a>>,
    blocks: Vec<Block>,
    statements: Vec<Statement>,
    last_statements: Vec<LastStatement>,
    expressions: Vec<Expression>,
    prefixes: Vec<Prefix>,
    arguments: Vec<Arguments>,
    variables: Vec<Variable>,
}

impl<'a> AstConverter<'a> {
    pub(crate) fn new(hold_token_data: bool) -> Self {
        Self {
            hold_token_data,
            ..Default::default()
        }
    }

    #[inline]
    fn push_work(&mut self, work: impl Into<ConvertWork<'a>>) {
        self.work_stack.push(work.into());
    }

    #[inline]
    fn pop_block(&mut self) -> Result<Block, ConvertError> {
        self.blocks
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Block" })
    }

    #[inline]
    fn pop_statement(&mut self) -> Result<Statement, ConvertError> {
        self.statements
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Statement" })
    }

    #[inline]
    fn pop_statements(&mut self, n: usize) -> Result<Vec<Statement>, ConvertError> {
        std::iter::repeat_with(|| self.pop_statement())
            .take(n)
            .collect()
    }

    #[inline]
    fn pop_last_statement(&mut self) -> Result<LastStatement, ConvertError> {
        self.last_statements
            .pop()
            .ok_or(ConvertError::InternalStack {
                kind: "LastStatement",
            })
    }

    #[inline]
    fn pop_expression(&mut self) -> Result<Expression, ConvertError> {
        self.expressions
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Expression" })
    }

    #[inline]
    fn pop_expressions(&mut self, n: usize) -> Result<Vec<Expression>, ConvertError> {
        std::iter::repeat_with(|| self.pop_expression())
            .take(n)
            .collect()
    }

    #[inline]
    fn pop_prefix(&mut self) -> Result<Prefix, ConvertError> {
        self.prefixes
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Prefix" })
    }

    #[inline]
    fn pop_variable(&mut self) -> Result<Variable, ConvertError> {
        self.variables
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Variable" })
    }

    #[inline]
    fn pop_variables(&mut self, n: usize) -> Result<Vec<Variable>, ConvertError> {
        std::iter::repeat_with(|| self.pop_variable())
            .take(n)
            .collect()
    }

    #[inline]
    fn pop_arguments(&mut self) -> Result<Arguments, ConvertError> {
        self.arguments
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Arguments" })
    }

    pub(crate) fn convert(&mut self, block: &'a ast::Block) -> Result<Block, ConvertError> {
        self.push_work(block);

        while let Some(work) = self.work_stack.pop() {
            match work {
                ConvertWork::PushVariable(variable) => {
                    self.variables.push(variable);
                }
                ConvertWork::PushExpression(expression) => {
                    self.expressions.push(expression);
                }
                ConvertWork::PushStatement(statement) => {
                    self.statements.push(statement);
                }
                ConvertWork::Block(block) => {
                    self.work_stack.push(ConvertWork::MakeBlock { block });
                    for stmt in block.stmts() {
                        self.push_work(stmt);
                    }
                    if let Some(last) = block.last_stmt() {
                        self.push_work(last);
                    }
                }
                ConvertWork::Statement(statement) => self.convert_statement(statement)?,
                ConvertWork::LastStatement(last_statement) => match last_statement {
                    ast::LastStmt::Break(token) => {
                        self.last_statements.push(if self.hold_token_data {
                            LastStatement::Break(Some(self.convert_token(token)?))
                        } else {
                            LastStatement::new_break()
                        });
                    }
                    ast::LastStmt::Continue(token) => {
                        self.last_statements.push(if self.hold_token_data {
                            LastStatement::Continue(Some(self.convert_token(token)?))
                        } else {
                            LastStatement::new_continue()
                        });
                    }
                    ast::LastStmt::Return(return_statement) => {
                        self.work_stack.push(ConvertWork::MakeReturn {
                            statement: return_statement,
                        });
                        for expression in return_statement.returns().iter() {
                            self.push_work(expression);
                        }
                    }
                    _ => {
                        return Err(ConvertError::LastStatement {
                            statement: last_statement.to_string(),
                        })
                    }
                },
                ConvertWork::Expression(expression) => self.convert_expression(expression)?,
                ConvertWork::Prefix(prefix) => match prefix {
                    ast::Prefix::Expression(expression) => {
                        self.work_stack
                            .push(ConvertWork::MakePrefixFromExpression { prefix });
                        self.push_work(expression);
                    }
                    ast::Prefix::Name(name) => {
                        self.prefixes
                            .push(self.convert_token_to_identifier(name)?.into());
                    }
                    _ => {
                        return Err(ConvertError::Prefix {
                            prefix: prefix.to_string(),
                        })
                    }
                },
                ConvertWork::Arguments(arguments) => match arguments {
                    ast::FunctionArgs::Parentheses {
                        parentheses,
                        arguments,
                    } => {
                        self.work_stack
                            .push(ConvertWork::MakeArgumentsFromExpressions {
                                parentheses,
                                arguments,
                            });
                        for value in arguments.iter() {
                            self.push_work(value);
                        }
                    }
                    ast::FunctionArgs::String(string) => {
                        self.arguments
                            .push(self.convert_string_expression(string)?.into());
                    }
                    ast::FunctionArgs::TableConstructor(table) => {
                        self.work_stack
                            .push(ConvertWork::MakeArgumentsFromTableEntries { table });

                        self.convert_table(table)?;
                    }
                    _ => {
                        return Err(ConvertError::FunctionArguments {
                            arguments: arguments.to_string(),
                        })
                    }
                },
                ConvertWork::MakeBlock { block } => {
                    let mut new_block = Block::new(
                        self.pop_statements(block.stmts().count())?,
                        block
                            .last_stmt()
                            .map(|_| self.pop_last_statement())
                            .transpose()?,
                    );

                    if self.hold_token_data {
                        let semicolons = block
                            .stmts_with_semicolon()
                            .map(|(_, token)| {
                                token
                                    .as_ref()
                                    .map(|token| self.convert_token(token))
                                    .transpose()
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let last_semicolon =
                            block.last_stmt_with_semicolon().and_then(|(_, semicolon)| {
                                semicolon.as_ref().map(|token| self.convert_token(token))
                            });

                        new_block.set_tokens(BlockTokens {
                            semicolons,
                            last_semicolon: last_semicolon.transpose()?,
                        });
                    };

                    self.blocks.push(new_block);
                }
                ConvertWork::MakeDoStatement { statement } => {
                    let block = self.pop_block()?;
                    let mut do_statement = DoStatement::new(block);
                    if self.hold_token_data {
                        do_statement.set_tokens(DoTokens {
                            r#do: self.convert_token(statement.do_token())?,
                            end: self.convert_token(statement.end_token())?,
                        })
                    }
                    self.statements.push(do_statement.into());
                }
                ConvertWork::MakeReturn { statement } => {
                    let mut return_statement =
                        ReturnStatement::new(self.pop_expressions(statement.returns().len())?);
                    if self.hold_token_data {
                        let commas = self.extract_tokens_from_punctuation(statement.returns())?;
                        return_statement.set_tokens(ReturnTokens {
                            r#return: self.convert_token(statement.token())?,
                            commas,
                        });
                    }
                    self.last_statements.push(return_statement.into());
                }
                ConvertWork::MakeBinaryExpression { operator } => {
                    let left = self.pop_expression()?;
                    let right = self.pop_expression()?;
                    let mut binary =
                        BinaryExpression::new(self.convert_binop(operator)?, left, right);
                    if self.hold_token_data {
                        binary.set_token(self.convert_token(get_binary_operator_token(operator)?)?);
                    }
                    self.expressions.push(binary.into());
                }
                ConvertWork::MakeUnaryExpression { operator } => {
                    let mut unary =
                        UnaryExpression::new(self.convert_unop(operator)?, self.pop_expression()?);
                    if self.hold_token_data {
                        unary.set_token(self.convert_token(get_unary_operator_token(operator)?)?);
                    }
                    self.expressions.push(unary.into());
                }
                ConvertWork::MakeParentheseExpression { contained_span } => {
                    let mut parenthese = ParentheseExpression::new(self.pop_expression()?);
                    if self.hold_token_data {
                        let (left, right) = contained_span.tokens();
                        parenthese.set_tokens(ParentheseTokens {
                            left_parenthese: self.convert_token(left)?,
                            right_parenthese: self.convert_token(right)?,
                        });
                    }
                    self.expressions.push(parenthese.into());
                }
                ConvertWork::MakeIfExpression { if_expression } => {
                    let condition = self.pop_expression()?;
                    let result = self.pop_expression()?;
                    let else_expression = self.pop_expression()?;

                    let mut value = IfExpression::new(condition, result, else_expression);

                    if let Some(elseifs) = if_expression.else_if_expressions() {
                        for elseif in elseifs.iter() {
                            let elseif_condition = self.pop_expression()?;
                            let elseif_expression = self.pop_expression()?;
                            let mut branch =
                                ElseIfExpressionBranch::new(elseif_condition, elseif_expression);
                            if self.hold_token_data {
                                branch.set_tokens(ElseIfExpressionBranchTokens {
                                    elseif: self.convert_token(elseif.else_if_token())?,
                                    then: self.convert_token(elseif.then_token())?,
                                });
                            }
                            value.push_branch(branch);
                        }
                    }

                    if self.hold_token_data {
                        value.set_tokens(IfExpressionTokens {
                            r#if: self.convert_token(if_expression.if_token())?,
                            then: self.convert_token(if_expression.then_token())?,
                            r#else: self.convert_token(if_expression.else_token())?,
                        });
                    }

                    self.expressions.push(value.into());
                }
                ConvertWork::MakeInterpolatedString {
                    interpolated_string,
                } => {
                    let mut segments = Vec::new();
                    let mut segments_iter = interpolated_string.segments().peekable();

                    while let Some(segment) = segments_iter.next() {
                        let literal = &segment.literal;
                        if let Some(segment) = self.convert_string_interpolation_segment(literal)? {
                            segments.push(segment.into());
                        }

                        let expression = self.pop_expression()?;
                        let mut value_segment = ValueSegment::new(expression);

                        if self.hold_token_data {
                            let opening_brace = Token::new_with_line(
                                literal.end_position().bytes().saturating_sub(1),
                                literal.end_position().bytes(),
                                literal.end_position().line(),
                            );

                            let next_literal = segments_iter
                                .peek()
                                .map(|next_segment| &next_segment.literal)
                                .unwrap_or(interpolated_string.last_string());

                            let start_position = next_literal.start_position().bytes();
                            let closing_brace = Token::new_with_line(
                                start_position,
                                start_position + 1,
                                next_literal.start_position().line(),
                            );

                            value_segment.set_tokens(ValueSegmentTokens {
                                opening_brace,
                                closing_brace,
                            });
                        }

                        segments.push(value_segment.into());
                    }

                    if let Some(segment) = self
                        .convert_string_interpolation_segment(interpolated_string.last_string())?
                    {
                        segments.push(segment.into());
                    }

                    let mut value = InterpolatedStringExpression::new(segments);

                    if self.hold_token_data {
                        let last = interpolated_string.last_string();
                        let first = interpolated_string
                            .segments()
                            .next()
                            .map(|segment| &segment.literal)
                            .unwrap_or(last);

                        let (opening_tick, closing_tick) = match first.token_type() {
                            TokenType::InterpolatedString { literal: _, kind } => match kind {
                                InterpolatedStringKind::Begin | InterpolatedStringKind::Simple => {
                                    let start_position = first.start_position().bytes();
                                    let mut start_token = Token::new_with_line(
                                        start_position,
                                        start_position + 1,
                                        first.start_position().line(),
                                    );
                                    let end_position = last.end_position().bytes();
                                    let mut end_token = Token::new_with_line(
                                        end_position.saturating_sub(1),
                                        end_position,
                                        last.end_position().line(),
                                    );

                                    for trivia_token in first.leading_trivia() {
                                        start_token.push_leading_trivia(
                                            self.convert_trivia(trivia_token)?,
                                        );
                                    }

                                    for trivia_token in first.trailing_trivia() {
                                        end_token.push_trailing_trivia(
                                            self.convert_trivia(trivia_token)?,
                                        );
                                    }
                                    (start_token, end_token)
                                }
                                InterpolatedStringKind::Middle | InterpolatedStringKind::End => {
                                    return Err(ConvertError::InterpolatedString {
                                        string: interpolated_string.to_string(),
                                    })
                                }
                            },
                            _ => {
                                return Err(ConvertError::InterpolatedString {
                                    string: interpolated_string.to_string(),
                                })
                            }
                        };

                        let tokens = InterpolatedStringTokens {
                            opening_tick,
                            closing_tick,
                        };
                        value.set_tokens(tokens);
                    }

                    self.expressions.push(value.into());
                }
                ConvertWork::MakeFunctionExpression { body, token } => {
                    let block = self.pop_block()?;
                    let (parameters, is_variadic, tokens) =
                        self.convert_function_body_attributes(body)?;

                    let mut function = FunctionExpression::new(block, parameters, is_variadic);

                    if let Some(tokens) = tokens {
                        function.set_tokens(FunctionExpressionTokens {
                            function: self.convert_token(token)?,
                            opening_parenthese: tokens.opening_parenthese,
                            closing_parenthese: tokens.closing_parenthese,
                            end: tokens.end,
                            parameter_commas: tokens.parameter_commas,
                            variable_arguments: tokens.variable_arguments,
                        })
                    }
                    self.expressions.push(function.into());
                }
                ConvertWork::MakeRepeatStatement { statement } => {
                    let mut repeat_statement =
                        RepeatStatement::new(self.pop_block()?, self.pop_expression()?);
                    if self.hold_token_data {
                        repeat_statement.set_tokens(RepeatTokens {
                            repeat: self.convert_token(statement.repeat_token())?,
                            until: self.convert_token(statement.until_token())?,
                        });
                    }
                    self.statements.push(repeat_statement.into());
                }
                ConvertWork::MakeWhileStatement { statement } => {
                    let block = self.pop_block()?;
                    let mut while_statement = WhileStatement::new(block, self.pop_expression()?);
                    if self.hold_token_data {
                        while_statement.set_tokens(WhileTokens {
                            r#while: self.convert_token(statement.while_token())?,
                            r#do: self.convert_token(statement.do_token())?,
                            end: self.convert_token(statement.end_token())?,
                        });
                    }
                    self.statements.push(while_statement.into());
                }
                ConvertWork::MakeNumericForStatement { statement } => {
                    let mut numeric_for = NumericForStatement::new(
                        self.convert_token_to_identifier(statement.index_variable())?,
                        self.pop_expression()?,
                        self.pop_expression()?,
                        statement
                            .step()
                            .map(|_| self.pop_expression())
                            .transpose()?,
                        self.pop_block()?,
                    );
                    if self.hold_token_data {
                        numeric_for.set_tokens(NumericForTokens {
                            r#for: self.convert_token(statement.for_token())?,
                            equal: self.convert_token(statement.equal_token())?,
                            r#do: self.convert_token(statement.do_token())?,
                            end: self.convert_token(statement.end_token())?,
                            end_comma: self.convert_token(statement.start_end_comma())?,
                            step_comma: statement
                                .end_step_comma()
                                .map(|token| self.convert_token(token))
                                .transpose()?,
                        });
                    }
                    self.statements.push(numeric_for.into());
                }
                ConvertWork::MakeGenericForStatement { statement } => {
                    let mut generic_for = GenericForStatement::new(
                        statement
                            .names()
                            .iter()
                            .map(|name| self.convert_token_to_identifier(name))
                            .collect::<Result<Vec<_>, _>>()?,
                        self.pop_expressions(statement.expressions().len())?,
                        self.pop_block()?,
                    );
                    if self.hold_token_data {
                        generic_for.set_tokens(GenericForTokens {
                            r#for: self.convert_token(statement.for_token())?,
                            r#in: self.convert_token(statement.in_token())?,
                            r#do: self.convert_token(statement.do_token())?,
                            end: self.convert_token(statement.end_token())?,
                            identifier_commas: self
                                .extract_tokens_from_punctuation(statement.names())?,
                            value_commas: self
                                .extract_tokens_from_punctuation(statement.expressions())?,
                        });
                    }
                    self.statements.push(generic_for.into());
                }
                ConvertWork::MakeFunctionDeclaration { statement } => {
                    let (parameters, is_variadic, tokens) =
                        self.convert_function_body_attributes(statement.body())?;
                    let name = self.convert_function_name(statement.name())?;
                    let mut function =
                        FunctionStatement::new(name, self.pop_block()?, parameters, is_variadic);

                    if let Some(tokens) = tokens {
                        function.set_tokens(FunctionStatementTokens {
                            function: self.convert_token(statement.function_token())?,
                            opening_parenthese: tokens.opening_parenthese,
                            closing_parenthese: tokens.closing_parenthese,
                            end: tokens.end,
                            parameter_commas: tokens.parameter_commas,
                            variable_arguments: tokens.variable_arguments,
                        });
                    }
                    self.statements.push(function.into());
                }
                ConvertWork::MakeFunctionCallStatement { call } => {
                    let call = self.make_function_call(call)?;
                    self.statements.push(call.into());
                }
                ConvertWork::MakePrefixFromExpression { prefix } => match self.pop_expression()? {
                    Expression::Parenthese(parenthese) => {
                        self.prefixes.push(Prefix::Parenthese(*parenthese));
                    }
                    _ => {
                        return Err(ConvertError::Prefix {
                            prefix: prefix.to_string(),
                        })
                    }
                },
                ConvertWork::MakeFunctionCallExpression { call } => {
                    let call = self.make_function_call(call)?;
                    self.expressions.push(call.into());
                }
                ConvertWork::MakeLocalFunctionStatement { statement } => {
                    let (parameters, is_variadic, tokens) =
                        self.convert_function_body_attributes(statement.body())?;
                    let mut name = Identifier::new(statement.name().token().to_string());
                    if self.hold_token_data {
                        name.set_token(self.convert_token(statement.name())?);
                    }
                    let mut local_function = LocalFunctionStatement::new(
                        name,
                        self.pop_block()?,
                        parameters,
                        is_variadic,
                    );
                    if let Some(tokens) = tokens {
                        local_function.set_tokens(LocalFunctionTokens {
                            local: self.convert_token(statement.local_token())?,
                            function: self.convert_token(statement.function_token())?,
                            opening_parenthese: tokens.opening_parenthese,
                            closing_parenthese: tokens.closing_parenthese,
                            end: tokens.end,
                            parameter_commas: tokens.parameter_commas,
                            variable_arguments: tokens.variable_arguments,
                        });
                    }
                    self.statements.push(local_function.into());
                }
                ConvertWork::MakeLocalAssignStatement { statement } => {
                    let variables = statement
                        .names()
                        .iter()
                        .map(|token_ref| self.convert_token_to_identifier(token_ref))
                        .collect::<Result<Vec<_>, _>>()?;

                    let mut local_assign = LocalAssignStatement::new(
                        variables,
                        self.pop_expressions(statement.expressions().len())?,
                    );

                    if self.hold_token_data {
                        local_assign.set_tokens(LocalAssignTokens {
                            local: self.convert_token(statement.local_token())?,
                            equal: statement
                                .equal_token()
                                .map(|token| self.convert_token(token))
                                .transpose()?,
                            variable_commas: self
                                .extract_tokens_from_punctuation(statement.names())?,
                            value_commas: self
                                .extract_tokens_from_punctuation(statement.expressions())?,
                        })
                    }
                    self.statements.push(local_assign.into());
                }
                ConvertWork::MakeArgumentsFromExpressions {
                    arguments,
                    parentheses,
                } => {
                    let mut tuple = TupleArguments::new(self.pop_expressions(arguments.len())?);
                    if self.hold_token_data {
                        let (left, right) = parentheses.tokens();
                        tuple.set_tokens(TupleArgumentsTokens {
                            opening_parenthese: self.convert_token(left)?,
                            closing_parenthese: self.convert_token(right)?,
                            commas: self.extract_tokens_from_punctuation(arguments)?,
                        })
                    }
                    self.arguments.push(tuple.into());
                }
                ConvertWork::MakeArgumentsFromTableEntries { table } => {
                    let expression = self.make_table_expression(table)?;
                    self.arguments.push(expression.into());
                }
                ConvertWork::MakeTableExpression { table } => {
                    let expression = self.make_table_expression(table)?;
                    self.expressions.push(expression.into());
                }
                ConvertWork::MakeAssignStatement { statement } => {
                    let variables = self.pop_variables(statement.variables().len())?;
                    let values = self.pop_expressions(statement.expressions().len())?;
                    let mut assignment = AssignStatement::new(variables, values);
                    if self.hold_token_data {
                        assignment.set_tokens(AssignTokens {
                            equal: self.convert_token(statement.equal_token())?,
                            variable_commas: self
                                .extract_tokens_from_punctuation(statement.variables())?,
                            value_commas: self
                                .extract_tokens_from_punctuation(statement.expressions())?,
                        });
                    }
                    self.statements.push(assignment.into());
                }
                ConvertWork::MakeVariable { variable } => {
                    let prefix = self.make_prefix_with_suffixes(variable.suffixes())?;
                    let variable = match prefix {
                        Prefix::Identifier(name) => Variable::Identifier(name),
                        Prefix::Field(field) => Variable::Field(field),
                        Prefix::Index(index) => Variable::Index(index),
                        Prefix::Call(_) | Prefix::Parenthese(_) => {
                            return Err(ConvertError::Variable {
                                variable: variable.to_string(),
                            })
                        }
                    };
                    self.variables.push(variable);
                }
                ConvertWork::MakePrefixExpression { variable } => {
                    let prefix = self.make_prefix_with_suffixes(variable.suffixes())?;
                    self.expressions.push(prefix.into());
                }
                ConvertWork::MakeCompoundAssignStatement { statement } => {
                    let variable = self.pop_variable()?;
                    let value = self.pop_expression()?;
                    let mut assignment = CompoundAssignStatement::new(
                        self.convert_compound_op(statement.compound_operator())?,
                        variable,
                        value,
                    );
                    if self.hold_token_data {
                        assignment.set_tokens(CompoundAssignTokens {
                            operator: self.convert_token(get_compound_operator_token(
                                statement.compound_operator(),
                            )?)?,
                        });
                    }
                    self.statements.push(assignment.into());
                }
                ConvertWork::MakeIfStatement { statement } => {
                    let condition = self.pop_expression()?;
                    let block = self.pop_block()?;
                    let mut if_statement = IfStatement::create(condition, block);
                    if let Some(elseifs) = statement.else_if() {
                        for else_if in elseifs {
                            let elseif_condition = self.pop_expression()?;
                            let elseif_block = self.pop_block()?;
                            let mut branch = IfBranch::new(elseif_condition, elseif_block);
                            if self.hold_token_data {
                                branch.set_tokens(IfBranchTokens {
                                    elseif: self.convert_token(else_if.else_if_token())?,
                                    then: self.convert_token(else_if.then_token())?,
                                });
                            }
                            if_statement.push_branch(branch);
                        }
                    }
                    if statement.else_block().is_some() {
                        if_statement.set_else_block(self.pop_block()?);
                    }
                    if self.hold_token_data {
                        if_statement.set_tokens(IfStatementTokens {
                            r#if: self.convert_token(statement.if_token())?,
                            then: self.convert_token(statement.then_token())?,
                            end: self.convert_token(statement.end_token())?,
                            r#else: statement
                                .else_token()
                                .map(|token| self.convert_token(token))
                                .transpose()?,
                        })
                    }
                    self.statements.push(if_statement.into());
                }
            }
        }

        let block = self.blocks.pop().expect("root block should be converted");

        Ok(block)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_statement(&mut self, statement: &'a ast::Stmt) -> Result<(), ConvertError> {
        match statement {
            ast::Stmt::Assignment(assignment) => {
                self.work_stack.push(ConvertWork::MakeAssignStatement {
                    statement: assignment,
                });
                for variable in assignment.variables() {
                    self.convert_variable(variable)?;
                }
                for expression in assignment.expressions() {
                    self.push_work(expression);
                }
            }
            ast::Stmt::Do(do_statement) => {
                self.work_stack.push(ConvertWork::MakeDoStatement {
                    statement: do_statement,
                });
                self.push_work(do_statement.block());
            }
            ast::Stmt::FunctionCall(call) => {
                self.work_stack
                    .push(ConvertWork::MakeFunctionCallStatement { call });
                self.convert_function_call(call)?;
            }
            ast::Stmt::FunctionDeclaration(function) => {
                self.work_stack.push(ConvertWork::MakeFunctionDeclaration {
                    statement: function,
                });
                self.push_work(function.body().block());
            }
            ast::Stmt::GenericFor(generic_for) => {
                self.work_stack.push(ConvertWork::MakeGenericForStatement {
                    statement: generic_for,
                });
                self.push_work(generic_for.block());
                for expression in generic_for.expressions().iter() {
                    self.push_work(expression);
                }
            }
            ast::Stmt::If(if_statement) => {
                self.work_stack.push(ConvertWork::MakeIfStatement {
                    statement: if_statement,
                });
                self.push_work(if_statement.condition());
                self.push_work(if_statement.block());
                if let Some(elseifs) = if_statement.else_if() {
                    for branch in elseifs {
                        self.push_work(branch.condition());
                        self.push_work(branch.block());
                    }
                }
                if let Some(block) = if_statement.else_block() {
                    self.push_work(block);
                }
            }
            ast::Stmt::LocalAssignment(local_assign) => {
                self.work_stack.push(ConvertWork::MakeLocalAssignStatement {
                    statement: local_assign,
                });
                for expression in local_assign.expressions().iter() {
                    self.push_work(expression);
                }
            }
            ast::Stmt::LocalFunction(local_function) => {
                self.work_stack
                    .push(ConvertWork::MakeLocalFunctionStatement {
                        statement: local_function,
                    });
                self.push_work(local_function.body().block());
            }
            ast::Stmt::NumericFor(numeric_for) => {
                self.work_stack.push(ConvertWork::MakeNumericForStatement {
                    statement: numeric_for,
                });
                self.push_work(numeric_for.block());
                self.work_stack
                    .push(ConvertWork::Expression(numeric_for.start()));
                self.work_stack
                    .push(ConvertWork::Expression(numeric_for.end()));
                if let Some(step) = numeric_for.step() {
                    self.push_work(step);
                }
            }
            ast::Stmt::Repeat(repeat) => {
                self.work_stack
                    .push(ConvertWork::MakeRepeatStatement { statement: repeat });
                self.push_work(repeat.block());
                self.push_work(repeat.until());
            }
            ast::Stmt::While(while_statement) => {
                self.work_stack.push(ConvertWork::MakeWhileStatement {
                    statement: while_statement,
                });
                self.push_work(while_statement.block());
                self.push_work(while_statement.condition());
            }
            ast::Stmt::CompoundAssignment(assignment) => {
                self.work_stack
                    .push(ConvertWork::MakeCompoundAssignStatement {
                        statement: assignment,
                    });
                self.convert_variable(assignment.lhs())?;
                self.push_work(assignment.rhs());
            }
            ast::Stmt::ExportedTypeDeclaration(_) => {
                // todo!()
                self.work_stack
                    .push(ConvertWork::PushStatement(DoStatement::default().into()));
            }
            ast::Stmt::TypeDeclaration(_) => {
                // todo!()
                self.work_stack
                    .push(ConvertWork::PushStatement(DoStatement::default().into()));
            }
            _ => {
                return Err(ConvertError::Statement {
                    statement: statement.to_string(),
                })
            }
        }
        Ok(())
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_table(&mut self, table: &'a ast::TableConstructor) -> Result<(), ConvertError> {
        for field in table.fields() {
            match field {
                ast::Field::ExpressionKey {
                    brackets: _,
                    key,
                    equal: _,
                    value,
                } => {
                    self.push_work(key);
                    self.push_work(value);
                }
                ast::Field::NameKey {
                    key: _,
                    equal: _,
                    value,
                } => {
                    self.push_work(value);
                }
                ast::Field::NoKey(value) => {
                    self.push_work(value);
                }
                _ => {
                    return Err(ConvertError::TableEntry {
                        entry: field.to_string(),
                    })
                }
            }
        }
        Ok(())
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn make_table_expression(
        &mut self,
        table: &ast::TableConstructor,
    ) -> Result<TableExpression, ConvertError> {
        let entries: Result<_, _> = table
            .fields()
            .iter()
            .map(|field| match field {
                ast::Field::ExpressionKey {
                    brackets,
                    key: _,
                    equal,
                    value: _,
                } => {
                    let key = self.pop_expression()?;
                    let value = self.pop_expression()?;
                    let mut entry = TableIndexEntry::new(key, value);
                    if self.hold_token_data {
                        let (left, right) = brackets.tokens();
                        entry.set_tokens(TableIndexEntryTokens {
                            opening_bracket: self.convert_token(left)?,
                            closing_bracket: self.convert_token(right)?,
                            equal: self.convert_token(equal)?,
                        })
                    }
                    Ok(entry.into())
                }
                ast::Field::NameKey {
                    key,
                    equal,
                    value: _,
                } => {
                    let mut entry = TableFieldEntry::new(
                        self.convert_token_to_identifier(key)?,
                        self.pop_expression()?,
                    );
                    if self.hold_token_data {
                        entry.set_token(self.convert_token(equal)?);
                    }
                    Ok(entry.into())
                }
                ast::Field::NoKey(_) => Ok(TableEntry::Value(self.pop_expression()?)),
                _ => Err(ConvertError::TableEntry {
                    entry: field.to_string(),
                }),
            })
            .collect();
        let mut expression = TableExpression::new(entries?);
        if self.hold_token_data {
            let (left, right) = table.braces().tokens();
            expression.set_tokens(TableTokens {
                opening_brace: self.convert_token(left)?,
                closing_brace: self.convert_token(right)?,
                separators: self.extract_tokens_from_punctuation(table.fields())?,
            });
        }
        Ok(expression)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn make_function_call(
        &mut self,
        call: &'a ast::FunctionCall,
    ) -> Result<FunctionCall, ConvertError> {
        let prefix = self.make_prefix_with_suffixes(call.suffixes())?;
        match prefix {
            Prefix::Call(call) => Ok(call),
            _ => panic!(
                "FunctionCall should convert to a call statement, but got {:#?}",
                prefix,
            ),
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn make_prefix_with_suffixes(
        &mut self,
        suffixes: impl Iterator<Item = &'a ast::Suffix>,
    ) -> Result<Prefix, ConvertError> {
        let mut prefix = self.pop_prefix()?;

        for suffix in suffixes {
            match suffix {
                ast::Suffix::Call(call_suffix) => match call_suffix {
                    ast::Call::AnonymousCall(_) => {
                        let mut call = FunctionCall::new(prefix, self.pop_arguments()?, None);
                        if self.hold_token_data {
                            call.set_tokens(FunctionCallTokens { colon: None })
                        }
                        prefix = call.into();
                    }
                    ast::Call::MethodCall(method_call) => {
                        let mut call = FunctionCall::new(
                            prefix,
                            self.pop_arguments()?,
                            Some(self.convert_token_to_identifier(method_call.name())?),
                        );
                        if self.hold_token_data {
                            call.set_tokens(FunctionCallTokens {
                                colon: Some(self.convert_token(method_call.colon_token())?),
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
                        expression: _,
                    } => {
                        let mut index = IndexExpression::new(prefix, self.pop_expression()?);
                        if self.hold_token_data {
                            let (left, right) = brackets.tokens();
                            index.set_tokens(IndexExpressionTokens {
                                opening_bracket: self.convert_token(left)?,
                                closing_bracket: self.convert_token(right)?,
                            });
                        }
                        prefix = index.into();
                    }
                    ast::Index::Dot { name, dot } => {
                        let mut field =
                            FieldExpression::new(prefix, self.convert_token_to_identifier(name)?);
                        if self.hold_token_data {
                            field.set_token(self.convert_token(dot)?);
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
    fn convert_expression(&mut self, expression: &'a ast::Expression) -> Result<(), ConvertError> {
        match expression {
            ast::Expression::BinaryOperator { lhs, binop, rhs } => {
                self.work_stack
                    .push(ConvertWork::MakeBinaryExpression { operator: binop });
                self.work_stack.push(ConvertWork::Expression(lhs));
                self.work_stack.push(ConvertWork::Expression(rhs));
            }
            ast::Expression::Parentheses {
                contained,
                expression: inner_expression,
            } => {
                self.work_stack.push(ConvertWork::MakeParentheseExpression {
                    contained_span: contained,
                });
                self.work_stack
                    .push(ConvertWork::Expression(inner_expression));
            }
            ast::Expression::UnaryOperator { unop, expression } => {
                self.work_stack
                    .push(ConvertWork::MakeUnaryExpression { operator: unop });
                self.work_stack.push(ConvertWork::Expression(expression));
            }
            ast::Expression::Value {
                value,
                type_assertion: _,
            } => match value.as_ref() {
                ast::Value::Function((token, body)) => {
                    self.work_stack
                        .push(ConvertWork::MakeFunctionExpression { body, token });
                    self.push_work(body.block());
                }
                ast::Value::FunctionCall(call) => {
                    self.work_stack
                        .push(ConvertWork::MakeFunctionCallExpression { call });
                    self.convert_function_call(call)?;
                }
                ast::Value::TableConstructor(table) => {
                    self.work_stack
                        .push(ConvertWork::MakeTableExpression { table });
                    self.convert_table(table)?;
                }
                ast::Value::Number(number) => {
                    let mut expression = NumberExpression::from_str(&number.token().to_string())
                        .map_err(|err| ConvertError::Number {
                            number: number.to_string(),
                            parsing_error: err.to_string(),
                        })?;
                    if self.hold_token_data {
                        expression.set_token(self.convert_token(number)?);
                    }
                    self.work_stack
                        .push(ConvertWork::PushExpression(expression.into()));
                }
                ast::Value::ParenthesesExpression(expression) => {
                    self.push_work(expression);
                }
                ast::Value::String(token_ref) => {
                    self.work_stack.push(ConvertWork::PushExpression(
                        self.convert_string_expression(token_ref)?.into(),
                    ));
                }
                ast::Value::Symbol(symbol_token) => match symbol_token.token().token_type() {
                    TokenType::Symbol { symbol } => {
                        let token = if self.hold_token_data {
                            Some(self.convert_token(symbol_token)?)
                        } else {
                            None
                        };
                        let expression = match symbol {
                            Symbol::True => Expression::True(token),
                            Symbol::False => Expression::False(token),
                            Symbol::Nil => Expression::Nil(token),
                            Symbol::Ellipse => Expression::VariableArguments(token),
                            _ => {
                                return Err(ConvertError::Expression {
                                    expression: expression.to_string(),
                                })
                            }
                        };
                        self.work_stack
                            .push(ConvertWork::PushExpression(expression));
                    }
                    _ => {
                        return Err(ConvertError::Expression {
                            expression: expression.to_string(),
                        })
                    }
                },
                ast::Value::Var(var) => match var {
                    ast::Var::Expression(var_expression) => {
                        self.work_stack.push(ConvertWork::MakePrefixExpression {
                            variable: var_expression,
                        });
                        self.push_work(var_expression.prefix());
                        self.convert_suffixes(var_expression.suffixes())?;
                    }
                    ast::Var::Name(token_ref) => {
                        self.work_stack
                            .push(ConvertWork::PushExpression(Expression::Identifier(
                                self.convert_token_to_identifier(token_ref)?,
                            )));
                    }
                    _ => {
                        return Err(ConvertError::Expression {
                            expression: expression.to_string(),
                        })
                    }
                },
                ast::Value::IfExpression(if_expression) => {
                    self.push_work(ConvertWork::MakeIfExpression { if_expression });
                    self.push_work(if_expression.condition());
                    self.push_work(if_expression.if_expression());
                    self.push_work(if_expression.else_expression());
                    if let Some(elseif_expressions) = if_expression.else_if_expressions() {
                        for elseif in elseif_expressions {
                            self.push_work(elseif.condition());
                            self.push_work(elseif.expression());
                        }
                    }
                }
                ast::Value::InterpolatedString(interpolated_string) => {
                    self.push_work(ConvertWork::MakeInterpolatedString {
                        interpolated_string,
                    });
                    for segment in interpolated_string.segments() {
                        self.push_work(&segment.expression);
                    }
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
        }
        Ok(())
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_function_call(&mut self, call: &'a ast::FunctionCall) -> Result<(), ConvertError> {
        self.push_work(call.prefix());
        self.convert_suffixes(call.suffixes())?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_suffixes(
        &mut self,
        suffixes: impl Iterator<Item = &'a ast::Suffix>,
    ) -> Result<(), ConvertError> {
        for suffix in suffixes {
            match suffix {
                ast::Suffix::Call(call_suffix) => match call_suffix {
                    ast::Call::AnonymousCall(arguments) => {
                        self.push_work(arguments);
                    }
                    ast::Call::MethodCall(method_call) => {
                        self.push_work(method_call.args());
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
                        self.push_work(expression);
                    }
                    ast::Index::Dot { name: _, dot: _ } => {}
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
        Ok(())
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_token(&self, token: &tokenizer::TokenReference) -> Result<Token, ConvertError> {
        let mut new_token = Token::new_with_line(
            token.start_position().bytes(),
            token.end_position().bytes(),
            token.start_position().line(),
        );

        for trivia_token in token.leading_trivia() {
            new_token.push_leading_trivia(self.convert_trivia(trivia_token)?);
        }

        for trivia_token in token.trailing_trivia() {
            new_token.push_trailing_trivia(self.convert_trivia(trivia_token)?);
        }

        Ok(new_token)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_trivia(&self, token: &tokenizer::Token) -> Result<Trivia, ConvertError> {
        use tokenizer::TokenKind;

        let trivia = match token.token_kind() {
            TokenKind::MultiLineComment => TriviaKind::Comment,
            TokenKind::SingleLineComment => TriviaKind::Comment,
            TokenKind::Whitespace => TriviaKind::Whitespace,
            _ => return Err(ConvertError::UnexpectedTrivia(token.token_kind())),
        }
        .at(
            token.start_position().bytes(),
            token.end_position().bytes(),
            token.start_position().line(),
        );
        Ok(trivia)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_token_to_identifier(
        &self,
        token: &tokenizer::TokenReference,
    ) -> Result<Identifier, ConvertError> {
        let mut identifier = Identifier::new(token.token().to_string());
        if self.hold_token_data {
            identifier.set_token(self.convert_token(token)?);
        }
        Ok(identifier)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn extract_tokens_from_punctuation<T>(
        &self,
        punctuated: &ast::punctuated::Punctuated<T>,
    ) -> Result<Vec<Token>, ConvertError> {
        punctuated
            .pairs()
            .filter_map(|pair| match pair {
                ast::punctuated::Pair::End(_) => None,
                ast::punctuated::Pair::Punctuated(_, token) => Some(self.convert_token(token)),
            })
            .collect()
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_function_body_attributes(
        &self,
        body: &ast::FunctionBody,
    ) -> Result<(Vec<Identifier>, bool, Option<FunctionBodyTokens>), ConvertError> {
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
                        identifier.set_token(self.convert_token(name)?);
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
            let commas = self.extract_tokens_from_punctuation(body.parameters())?;
            Some(FunctionBodyTokens {
                opening_parenthese: self.convert_token(open)?,
                closing_parenthese: self.convert_token(close)?,
                end: self.convert_token(body.end_token())?,
                parameter_commas: commas,
                variable_arguments: is_variadic
                    .map(|token| self.convert_token(token))
                    .transpose()?,
            })
        } else {
            None
        };

        Ok((parameters, is_variadic.is_some(), tokens))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_string_expression(
        &self,
        string: &tokenizer::TokenReference,
    ) -> Result<StringExpression, ConvertError> {
        let mut expression = StringExpression::new(&string.token().to_string())
            .expect("unable to convert string expression");
        if self.hold_token_data {
            expression.set_token(self.convert_token(string)?);
        }
        Ok(expression)
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
    fn convert_function_name(
        &self,
        name: &ast::FunctionName,
    ) -> Result<FunctionName, ConvertError> {
        let mut name_iter = name
            .names()
            .iter()
            .map(|token_ref| self.convert_token_to_identifier(token_ref));

        let mut function_name = FunctionName::new(
            name_iter
                .next()
                .transpose()?
                .ok_or(ConvertError::ExpectedFunctionName)?,
            name_iter.collect::<Result<Vec<_>, _>>()?,
            name.method_name()
                .map(|token_ref| self.convert_token_to_identifier(token_ref))
                .transpose()?,
        );

        if self.hold_token_data {
            function_name.set_tokens(FunctionNameTokens {
                periods: self.extract_tokens_from_punctuation(name.names())?,
                colon: name
                    .method_colon()
                    .map(|colon| self.convert_token(colon))
                    .transpose()?,
            });
        }

        Ok(function_name)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_variable(&mut self, variable: &'a ast::Var) -> Result<(), ConvertError> {
        match variable {
            ast::Var::Expression(var_expression) => {
                self.work_stack.push(ConvertWork::MakeVariable {
                    variable: var_expression,
                });
                self.push_work(var_expression.prefix());
                self.convert_suffixes(var_expression.suffixes())?;
            }
            ast::Var::Name(name) => {
                self.work_stack.push(ConvertWork::PushVariable(
                    self.convert_token_to_identifier(name)?.into(),
                ));
            }
            _ => {
                return Err(ConvertError::Variable {
                    variable: variable.to_string(),
                })
            }
        }
        Ok(())
    }

    fn convert_string_interpolation_segment(
        &self,
        token: &tokenizer::TokenReference,
    ) -> Result<Option<StringSegment>, ConvertError> {
        match token.token_type() {
            TokenType::InterpolatedString { literal, kind: _ } => {
                if !literal.is_empty() {
                    let mut segment = StringSegment::new(literal.as_str());

                    if self.hold_token_data {
                        let segment_token = Token::new_with_line(
                            token.start_position().bytes() + 1,
                            token.end_position().bytes().saturating_sub(1),
                            token.start_position().line(),
                        );
                        // no trivia since it is grabbing a substring of the token
                        segment.set_token(segment_token);
                    }

                    Ok(Some(segment))
                } else {
                    Ok(None)
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
enum ConvertWork<'a> {
    Block(&'a ast::Block),
    Statement(&'a ast::Stmt),
    LastStatement(&'a ast::LastStmt),
    Expression(&'a ast::Expression),
    Prefix(&'a ast::Prefix),
    Arguments(&'a ast::FunctionArgs),
    PushStatement(Statement),
    PushExpression(Expression),
    PushVariable(Variable),
    MakeBlock {
        block: &'a ast::Block,
    },
    MakeDoStatement {
        statement: &'a ast::Do,
    },
    MakeReturn {
        statement: &'a ast::Return,
    },
    MakeBinaryExpression {
        operator: &'a ast::BinOp,
    },
    MakeUnaryExpression {
        operator: &'a ast::UnOp,
    },
    MakeParentheseExpression {
        contained_span: &'a ast::span::ContainedSpan,
    },
    MakeIfExpression {
        if_expression: &'a ast::types::IfExpression,
    },
    MakeFunctionExpression {
        body: &'a ast::FunctionBody,
        token: &'a tokenizer::TokenReference,
    },
    MakeRepeatStatement {
        statement: &'a ast::Repeat,
    },
    MakeWhileStatement {
        statement: &'a ast::While,
    },
    MakeNumericForStatement {
        statement: &'a ast::NumericFor,
    },
    MakeGenericForStatement {
        statement: &'a ast::GenericFor,
    },
    MakeFunctionDeclaration {
        statement: &'a ast::FunctionDeclaration,
    },
    MakeFunctionCallExpression {
        call: &'a ast::FunctionCall,
    },
    MakeFunctionCallStatement {
        call: &'a ast::FunctionCall,
    },
    MakePrefixFromExpression {
        prefix: &'a ast::Prefix,
    },
    MakeLocalFunctionStatement {
        statement: &'a ast::LocalFunction,
    },
    MakeLocalAssignStatement {
        statement: &'a ast::LocalAssignment,
    },
    MakeAssignStatement {
        statement: &'a ast::Assignment,
    },
    MakeCompoundAssignStatement {
        statement: &'a ast::types::CompoundAssignment,
    },
    MakeIfStatement {
        statement: &'a ast::If,
    },
    MakeArgumentsFromExpressions {
        arguments: &'a ast::punctuated::Punctuated<ast::Expression>,
        parentheses: &'a ast::span::ContainedSpan,
    },
    MakeArgumentsFromTableEntries {
        table: &'a ast::TableConstructor,
    },
    MakeTableExpression {
        table: &'a ast::TableConstructor,
    },
    MakeVariable {
        variable: &'a ast::VarExpression,
    },
    MakePrefixExpression {
        variable: &'a ast::VarExpression,
    },
    MakeInterpolatedString {
        interpolated_string: &'a ast::types::InterpolatedString,
    },
}

impl<'a> From<&'a ast::Block> for ConvertWork<'a> {
    fn from(block: &'a ast::Block) -> Self {
        ConvertWork::Block(block)
    }
}

impl<'a> From<&'a ast::Stmt> for ConvertWork<'a> {
    fn from(statement: &'a ast::Stmt) -> Self {
        ConvertWork::Statement(statement)
    }
}

impl<'a> From<&'a ast::LastStmt> for ConvertWork<'a> {
    fn from(statement: &'a ast::LastStmt) -> Self {
        ConvertWork::LastStatement(statement)
    }
}

impl<'a> From<&'a ast::Expression> for ConvertWork<'a> {
    fn from(expression: &'a ast::Expression) -> Self {
        ConvertWork::Expression(expression)
    }
}

impl<'a> From<&'a ast::Prefix> for ConvertWork<'a> {
    fn from(prefix: &'a ast::Prefix) -> Self {
        ConvertWork::Prefix(prefix)
    }
}

impl<'a> From<&'a ast::FunctionArgs> for ConvertWork<'a> {
    fn from(arguments: &'a ast::FunctionArgs) -> Self {
        ConvertWork::Arguments(arguments)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ConvertError {
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
    InterpolatedString {
        string: String,
    },
    UnexpectedTrivia(tokenizer::TokenKind),
    ExpectedFunctionName,
    InternalStack {
        kind: &'static str,
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
            ConvertError::InterpolatedString { string } => ("interpolated string", string),
            ConvertError::Expression { expression } => ("expression", expression),
            ConvertError::FunctionParameter { parameter } => ("parameter", parameter),
            ConvertError::FunctionParameters { parameters } => ("parameters", parameters),
            ConvertError::TableEntry { entry } => ("table entry", entry),
            ConvertError::BinaryOperator { operator } => ("binary operator", operator),
            ConvertError::CompoundOperator { operator } => ("compound operator", operator),
            ConvertError::UnaryOperator { operator } => ("unary operator", operator),
            ConvertError::UnexpectedTrivia(token_kind) => {
                return write!(
                    f,
                    "unable to convert trivia from token kind `{:?}`",
                    token_kind
                );
            }
            ConvertError::ExpectedFunctionName => {
                return write!(f, "unable to convert empty function name",);
            }
            ConvertError::InternalStack { kind } => {
                return write!(
                    f,
                    "internal conversion stack expected to find an item of `{}`",
                    kind
                )
            }
        };
        write!(f, "unable to convert {} from `{}`", kind, code)
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

#[cfg(test)]
mod test {
    use super::*;

    mod convert_error {
        use super::*;

        #[test]
        fn display_unexpected_trivia_symbol() {
            assert_eq!(
                ConvertError::UnexpectedTrivia(tokenizer::TokenKind::Symbol).to_string(),
                "unable to convert trivia from token kind `Symbol`"
            )
        }

        #[test]
        fn display_unexpected_trivia_eof() {
            assert_eq!(
                ConvertError::UnexpectedTrivia(tokenizer::TokenKind::Eof).to_string(),
                "unable to convert trivia from token kind `Eof`"
            )
        }
    }
}
