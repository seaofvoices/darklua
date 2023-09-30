use std::{fmt, iter::FromIterator, str::FromStr};

use full_moon::{
    ast,
    tokenizer::{self, Symbol, TokenType},
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
    types: Vec<Type>,
    function_return_types: Vec<FunctionReturnType>,
    variadic_type_packs: Vec<VariadicTypePack>,
    generic_type_packs: Vec<GenericTypePack>,
    type_parameters: Vec<TypeParameters>,
    type_packs: Vec<TypePack>,
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

    #[inline]
    fn pop_type(&mut self) -> Result<Type, ConvertError> {
        self.types
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "Type" })
    }

    #[inline]
    fn pop_types(&mut self, n: usize) -> Result<Vec<Type>, ConvertError> {
        std::iter::repeat_with(|| self.pop_type()).take(n).collect()
    }

    #[inline]
    fn pop_variadic_type_pack(&mut self) -> Result<VariadicTypePack, ConvertError> {
        self.variadic_type_packs
            .pop()
            .ok_or(ConvertError::InternalStack {
                kind: "VariadicTypePack",
            })
    }

    #[inline]
    fn pop_generic_type_pack(&mut self) -> Result<GenericTypePack, ConvertError> {
        self.generic_type_packs
            .pop()
            .ok_or(ConvertError::InternalStack {
                kind: "GenericTypePack",
            })
    }

    #[inline]
    fn pop_function_return_type(&mut self) -> Result<FunctionReturnType, ConvertError> {
        self.function_return_types
            .pop()
            .ok_or(ConvertError::InternalStack {
                kind: "FunctionReturnType",
            })
    }

    #[inline]
    fn pop_type_parameters(&mut self) -> Result<TypeParameters, ConvertError> {
        self.type_parameters
            .pop()
            .ok_or(ConvertError::InternalStack {
                kind: "TypeParameters",
            })
    }

    #[inline]
    fn pop_type_pack(&mut self) -> Result<TypePack, ConvertError> {
        self.type_packs
            .pop()
            .ok_or(ConvertError::InternalStack { kind: "TypePack" })
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
                ConvertWork::PushType(r#type) => {
                    self.types.push(r#type);
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
                ConvertWork::TypeInfo(type_info) => self.convert_type_info(type_info)?,
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
                        let (left_parenthese, right_parenthese) =
                            self.extract_contained_span_tokens(contained_span)?;
                        parenthese.set_tokens(ParentheseTokens {
                            left_parenthese,
                            right_parenthese,
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
                ConvertWork::MakeFunctionExpression { body, token } => {
                    let builder =
                        self.convert_function_body_attributes(body, self.convert_token(token)?)?;

                    self.expressions
                        .push(builder.into_function_expression().into());
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
                    let typed_identifier = self.convert_typed_identifier(
                        statement.index_variable(),
                        statement.type_specifier(),
                    )?;

                    let block = self.pop_block()?;
                    let start = self.pop_expression()?;
                    let end = self.pop_expression()?;
                    let step = statement
                        .step()
                        .map(|_| self.pop_expression())
                        .transpose()?;

                    let mut numeric_for =
                        NumericForStatement::new(typed_identifier, start, end, step, block);

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
                    let block = self.pop_block()?;
                    let identifiers = statement
                        .names()
                        .iter()
                        .zip(statement.type_specifiers())
                        .map(|(name, type_specifier)| {
                            self.convert_typed_identifier(name, type_specifier)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let mut generic_for = GenericForStatement::new(
                        identifiers,
                        self.pop_expressions(statement.expressions().len())?,
                        block,
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
                    let builder = self.convert_function_body_attributes(
                        statement.body(),
                        self.convert_token(statement.function_token())?,
                    )?;
                    let name = self.convert_function_name(statement.name())?;

                    self.statements
                        .push(builder.into_function_statement(name).into());
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
                ConvertWork::MakeTypeDeclarationStatement {
                    type_declaration,
                    export_token,
                } => {
                    let mut declaration = TypeDeclarationStatement::new(
                        self.convert_token_to_identifier(type_declaration.type_name())?,
                        self.pop_type()?,
                    );

                    if export_token.is_some() {
                        declaration.set_exported();
                    }

                    if let Some(generics) = type_declaration.generics() {
                        let mut type_variables = Vec::new();
                        let mut generic_type_packs = Vec::new();
                        let mut type_variables_with_default = Vec::new();
                        let mut generic_type_packs_with_default = Vec::new();

                        for parameter in generics.generics() {
                            match parameter.parameter() {
                                ast::types::GenericParameterInfo::Name(token) => {
                                    let name = self.convert_token_to_identifier(token)?;

                                    if let Some(default_type) = parameter
                                        .default_type()
                                        .map(|_| self.pop_type())
                                        .transpose()?
                                    {
                                        type_variables_with_default.push(if self.hold_token_data {
                                            let equal_token =
                                                parameter.equals().ok_or_else(|| {
                                                    ConvertError::GenericDeclaration {
                                                        generics: generics.to_string(),
                                                    }
                                                })?;
                                            (
                                                name,
                                                default_type,
                                                Some(self.convert_token(equal_token)?),
                                            )
                                        } else {
                                            (name, default_type, None)
                                        });
                                    } else {
                                        type_variables
                                            .push(self.convert_token_to_identifier(token)?);
                                    }
                                }
                                ast::types::GenericParameterInfo::Variadic { name, ellipse } => {
                                    let mut generic_pack = GenericTypePack::new(
                                        self.convert_token_to_identifier(name)?,
                                    );

                                    if self.hold_token_data {
                                        generic_pack.set_token(self.convert_token(ellipse)?);
                                    }

                                    use ast::types::TypeInfo;

                                    if let Some(default_type) = parameter
                                        .default_type()
                                        .map(|default_type| match default_type {
                                            TypeInfo::GenericPack { .. } => self
                                                .pop_generic_type_pack()
                                                .map(GenericTypePackDefault::from),
                                            TypeInfo::VariadicPack { .. } => self
                                                .pop_variadic_type_pack()
                                                .map(GenericTypePackDefault::from),
                                            TypeInfo::Tuple { .. } => self
                                                .pop_type_pack()
                                                .map(GenericTypePackDefault::from),
                                            TypeInfo::Intersection { left, .. } | TypeInfo::Union { left, .. }
                                                if matches!(**left, TypeInfo::Variadic { .. }) =>
                                            {
                                                self.pop_variadic_type_pack().map(GenericTypePackDefault::from)
                                            }
                                            TypeInfo::Array { .. }
                                            | TypeInfo::Basic(_)
                                            | TypeInfo::String(_)
                                            | TypeInfo::Boolean(_)
                                            | TypeInfo::Callback { .. }
                                            | TypeInfo::Generic { .. }
                                            | TypeInfo::Module { .. }
                                            | TypeInfo::Optional { .. }
                                            | TypeInfo::Table { .. }
                                            | TypeInfo::Typeof { .. }
                                            | TypeInfo::Variadic { .. } // todo: maybe this needs to pop a variadic_type_pack?
                                            | _ => Err(ConvertError::GenericDeclaration {
                                                generics: generics.to_string(),
                                            }),
                                        })
                                        .transpose()?
                                    {
                                        let mut generic_pack_with_default =
                                            GenericTypePackWithDefault::new(
                                                generic_pack,
                                                default_type,
                                            );

                                        if self.hold_token_data {
                                            let equal_token =
                                                parameter.equals().ok_or_else(|| {
                                                    ConvertError::GenericDeclaration {
                                                        generics: generics.to_string(),
                                                    }
                                                })?;
                                            generic_pack_with_default
                                                .set_token(self.convert_token(equal_token)?);
                                        }

                                        generic_type_packs_with_default
                                            .push(generic_pack_with_default);
                                    } else {
                                        generic_type_packs.push(generic_pack)
                                    }
                                }
                                _ => {
                                    return Err(ConvertError::GenericDeclaration {
                                        generics: generics.to_string(),
                                    })
                                }
                            }
                        }

                        let mut type_variable_iter = type_variables.into_iter();
                        let mut type_variable_with_default_iter = type_variables_with_default
                            .into_iter()
                            .map(|(variable, default, token)| {
                                let mut type_variable =
                                    TypeVariableWithDefault::new(variable, default);

                                if let Some(token) = token {
                                    type_variable.set_token(token);
                                }

                                type_variable
                            });
                        let mut generic_type_packs_iter = generic_type_packs.into_iter();
                        let mut generic_type_packs_with_default_iter =
                            generic_type_packs_with_default.into_iter();

                        let mut generic_parameters = type_variable_iter
                            .next()
                            .map(GenericParametersWithDefaults::from_type_variable)
                            .or_else(|| {
                                type_variable_with_default_iter.next().map(
                                    GenericParametersWithDefaults::from_type_variable_with_default,
                                )
                            })
                            .or_else(|| {
                                generic_type_packs_iter
                                    .next()
                                    .map(GenericParametersWithDefaults::from_generic_type_pack)
                            })
                            .or_else(|| {
                                generic_type_packs_with_default_iter
                                    .next()
                                    .map(GenericParametersWithDefaults::from_generic_type_pack_with_default)
                            })
                            .ok_or_else(|| ConvertError::GenericDeclaration {
                                generics: generics.to_string(),
                            })?;

                        for type_variable in type_variable_iter {
                            generic_parameters.push_type_variable(type_variable);
                        }

                        for type_variable_with_default in type_variable_with_default_iter {
                            generic_parameters
                                .push_type_variable_with_default(type_variable_with_default);
                        }

                        for generic_type_pack in generic_type_packs_iter {
                            if !generic_parameters.push_generic_type_pack(generic_type_pack) {
                                return Err(ConvertError::GenericDeclaration {
                                    generics: generics.to_string(),
                                });
                            }
                        }

                        if self.hold_token_data {
                            let (opening_list, closing_list) =
                                self.extract_contained_span_tokens(generics.arrows())?;
                            generic_parameters.set_tokens(GenericParametersTokens {
                                opening_list,
                                closing_list,
                                commas: self
                                    .extract_tokens_from_punctuation(generics.generics())?,
                            });
                        }

                        declaration.set_generic_parameters(generic_parameters);
                    }

                    if self.hold_token_data {
                        declaration.set_tokens(TypeDeclarationTokens {
                            r#type: self.convert_token(type_declaration.type_token())?,
                            equal: self.convert_token(type_declaration.equal_token())?,
                            export: export_token
                                .map(|token| self.convert_token(token))
                                .transpose()?,
                        });
                    }
                    self.statements.push(declaration.into());
                }
                ConvertWork::MakeFunctionCallExpression { call } => {
                    let call = self.make_function_call(call)?;
                    self.expressions.push(call.into());
                }
                ConvertWork::MakeLocalFunctionStatement { statement } => {
                    let builder = self.convert_function_body_attributes(
                        statement.body(),
                        self.convert_token(statement.function_token())?,
                    )?;
                    let mut name = Identifier::new(statement.name().token().to_string());
                    let mut local_token = None;

                    if self.hold_token_data {
                        name.set_token(self.convert_token(statement.name())?);
                        local_token = Some(self.convert_token(statement.local_token())?);
                    }

                    self.statements.push(
                        builder
                            .into_local_function_statement(name, local_token)
                            .into(),
                    );
                }
                ConvertWork::MakeLocalAssignStatement { statement } => {
                    let variables = statement
                        .names()
                        .iter()
                        .zip(statement.type_specifiers())
                        .map(|(token_ref, type_specifier)| {
                            self.convert_typed_identifier(token_ref, type_specifier)
                        })
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
                        let (opening_parenthese, closing_parenthese) =
                            self.extract_contained_span_tokens(parentheses)?;
                        tuple.set_tokens(TupleArgumentsTokens {
                            opening_parenthese,
                            closing_parenthese,
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
                ConvertWork::MakeFunctionReturnType { type_info } => {
                    use ast::types::TypeInfo;
                    let return_type = match type_info {
                        TypeInfo::Tuple { .. } => self.pop_type_pack()?.into(),
                        TypeInfo::Variadic { .. } => self.pop_variadic_type_pack()?.into(),
                        TypeInfo::GenericPack { .. } => self.pop_generic_type_pack()?.into(),
                        TypeInfo::Intersection { left, .. } | TypeInfo::Union { left, .. }
                            if matches!(**left, TypeInfo::Variadic { .. }) =>
                        {
                            self.pop_variadic_type_pack()?.into()
                        }
                        _ => self.pop_type()?.into(),
                    };

                    self.function_return_types.push(return_type);
                }
                ConvertWork::MakeVariadicTypePack { ellipse } => {
                    let mut variadic_type_pack = VariadicTypePack::new(self.pop_type()?);

                    if self.hold_token_data {
                        variadic_type_pack.set_token(self.convert_token(ellipse)?);
                    }

                    self.variadic_type_packs.push(variadic_type_pack);
                }
                ConvertWork::MakeArrayType { braces } => {
                    let mut array_type = ArrayType::new(self.pop_type()?);

                    if self.hold_token_data {
                        let (opening_brace, closing_brace) =
                            self.extract_contained_span_tokens(braces)?;

                        array_type.set_tokens(ArrayTypeTokens {
                            opening_brace,
                            closing_brace,
                        })
                    }

                    self.types.push(array_type.into());
                }
                ConvertWork::MakeOptionalType { question_mark } => {
                    let mut optional_type = OptionalType::new(self.pop_type()?);

                    if self.hold_token_data {
                        optional_type.set_token(self.convert_token(question_mark)?);
                    }

                    self.types.push(optional_type.into());
                }
                ConvertWork::MakeIntersectionType { operator } => {
                    let left_type = self.pop_type()?;
                    let right_type = self.pop_type()?;

                    let mut intersection_type = IntersectionType::new(left_type, right_type);

                    if self.hold_token_data {
                        intersection_type.set_token(self.convert_token(operator)?);
                    }

                    self.types.push(intersection_type.into());
                }
                ConvertWork::MakeUnionType { operator } => {
                    let left_type = self.pop_type()?;
                    let right_type = self.pop_type()?;

                    let mut union_type = UnionType::new(left_type, right_type);

                    if self.hold_token_data {
                        union_type.set_token(self.convert_token(operator)?);
                    }

                    self.types.push(union_type.into());
                }
                ConvertWork::MakeTableType { braces, fields } => {
                    let mut table_type = TableType::default();

                    for field in fields {
                        use ast::types::TypeFieldKey;

                        match field.key() {
                            TypeFieldKey::Name(property_name) => {
                                let mut property_type = TablePropertyType::new(
                                    self.convert_token_to_identifier(property_name)?,
                                    self.pop_type()?,
                                );

                                if self.hold_token_data {
                                    property_type
                                        .set_token(self.convert_token(field.colon_token())?);
                                }

                                table_type.push_property(property_type);
                            }
                            TypeFieldKey::IndexSignature { brackets, .. } => {
                                let mut indexer_type =
                                    TableIndexerType::new(self.pop_type()?, self.pop_type()?);

                                if self.hold_token_data {
                                    let (opening_bracket, closing_bracket) =
                                        self.extract_contained_span_tokens(brackets)?;

                                    indexer_type.set_tokens(TableIndexerTypeTokens {
                                        opening_bracket,
                                        closing_bracket,
                                        colon: self.convert_token(field.colon_token())?,
                                    })
                                }

                                table_type.set_indexer_type(indexer_type);
                            }
                            key => {
                                return Err(ConvertError::TableTypeProperty {
                                    property: key.to_string(),
                                });
                            }
                        }
                    }

                    if self.hold_token_data {
                        let (opening_brace, closing_brace) =
                            self.extract_contained_span_tokens(braces)?;

                        table_type.set_tokens(TableTypeTokens {
                            opening_brace,
                            closing_brace,
                            separators: self.extract_tokens_from_punctuation(fields)?,
                        })
                    }

                    self.types.push(table_type.into());
                }
                ConvertWork::MakeExpressionType {
                    typeof_token,
                    parentheses,
                } => {
                    let mut expression_type = ExpressionType::new(self.pop_expression()?);

                    if self.hold_token_data {
                        let (opening_parenthese, closing_parenthese) =
                            self.extract_contained_span_tokens(parentheses)?;

                        expression_type.set_tokens(ExpressionTypeTokens {
                            r#typeof: self.convert_token(typeof_token)?,
                            opening_parenthese,
                            closing_parenthese,
                        });
                    }

                    self.types.push(expression_type.into());
                }
                ConvertWork::MakeFunctionType {
                    generics,
                    parentheses,
                    arguments,
                    arrow,
                } => {
                    let mut function_type = FunctionType::new(self.pop_function_return_type()?);

                    for argument in arguments {
                        use ast::types::TypeInfo;

                        match argument.type_info() {
                            TypeInfo::Variadic { .. } | TypeInfo::VariadicPack { .. } => {
                                function_type.set_variadic_type(self.pop_variadic_type_pack()?);
                            }
                            TypeInfo::GenericPack { .. } => {
                                function_type.set_variadic_type(self.pop_generic_type_pack()?);
                            }
                            TypeInfo::Intersection { .. }
                            | TypeInfo::Union { .. }
                            | TypeInfo::Array { .. }
                            | TypeInfo::Basic(_)
                            | TypeInfo::String(_)
                            | TypeInfo::Boolean(_)
                            | TypeInfo::Callback { .. }
                            | TypeInfo::Generic { .. }
                            | TypeInfo::Module { .. }
                            | TypeInfo::Optional { .. }
                            | TypeInfo::Table { .. }
                            | TypeInfo::Typeof { .. }
                            | TypeInfo::Tuple { .. }
                            | _ => {
                                let mut argument_type = FunctionArgumentType::new(self.pop_type()?);

                                if let Some((name, colon)) = argument.name() {
                                    argument_type.set_name(self.convert_token_to_identifier(name)?);

                                    if self.hold_token_data {
                                        argument_type.set_token(self.convert_token(colon)?);
                                    }
                                }

                                function_type.push_argument(argument_type);
                            }
                        };
                    }

                    if let Some(generics) = generics {
                        let generic_parameters = self.convert_generic_type_parameters(generics)?;

                        function_type.set_generic_parameters(generic_parameters);
                    }

                    if self.hold_token_data {
                        let (opening_parenthese, closing_parenthese) =
                            self.extract_contained_span_tokens(parentheses)?;

                        function_type.set_tokens(FunctionTypeTokens {
                            opening_parenthese,
                            closing_parenthese,
                            arrow: self.convert_token(arrow)?,
                            commas: self.extract_tokens_from_punctuation(arguments)?,
                        });
                    }

                    self.types.push(function_type.into());
                }
                ConvertWork::MakeGenericType { base, module } => {
                    let type_name = TypeName::new(self.convert_token_to_identifier(base)?)
                        .with_type_parameters(self.pop_type_parameters()?);

                    self.types
                        .push(if let Some((module, punctuation)) = module {
                            let mut type_field = TypeField::new(
                                self.convert_token_to_identifier(module)?,
                                type_name,
                            );

                            if self.hold_token_data {
                                type_field.set_token(self.convert_token(punctuation)?);
                            }

                            type_field.into()
                        } else {
                            type_name.into()
                        });
                }
                ConvertWork::MakeTypeParameters { arrows, generics } => {
                    use ast::types::TypeInfo;

                    let mut parameters = generics
                        .iter()
                        .map(|type_parameter| match type_parameter {
                            TypeInfo::GenericPack { .. } => {
                                self.pop_generic_type_pack().map(TypeParameter::from)
                            }
                            TypeInfo::VariadicPack { .. } | TypeInfo::Variadic { .. } => {
                                self.pop_variadic_type_pack().map(TypeParameter::from)
                            }
                            TypeInfo::Tuple { .. } => self.pop_type_pack().map(TypeParameter::from),
                            TypeInfo::Array { .. }
                            | TypeInfo::Basic(_)
                            | TypeInfo::String(_)
                            | TypeInfo::Boolean(_)
                            | TypeInfo::Callback { .. }
                            | TypeInfo::Generic { .. }
                            | TypeInfo::Intersection { .. }
                            | TypeInfo::Module { .. }
                            | TypeInfo::Optional { .. }
                            | TypeInfo::Table { .. }
                            | TypeInfo::Typeof { .. }
                            | TypeInfo::Union { .. } => self.pop_type().map(TypeParameter::from),
                            _ => Err(ConvertError::TypeInfo {
                                type_info: type_parameter.to_string(),
                            }),
                        })
                        .collect::<Result<TypeParameters, ConvertError>>()?;

                    if self.hold_token_data {
                        let (opening_list, closing_list) =
                            self.extract_contained_span_tokens(arrows)?;

                        let commas = self.extract_tokens_from_punctuation(generics)?;

                        parameters.set_tokens(TypeParametersTokens {
                            opening_list,
                            closing_list,
                            commas,
                        })
                    }

                    self.type_parameters.push(parameters);
                }
                ConvertWork::MakeTypeCast { type_assertion } => {
                    let r#type = self.pop_type()?;
                    let expression = self.pop_expression()?;

                    let mut type_cast = TypeCastExpression::new(expression, r#type);

                    if self.hold_token_data {
                        type_cast.set_token(self.convert_token(type_assertion.assertion_op())?);
                    }

                    self.expressions.push(type_cast.into());
                }
                ConvertWork::MakeParentheseType { parentheses } => {
                    let r#type = self.pop_type()?;

                    let mut parenthese_type = ParentheseType::new(r#type);

                    if self.hold_token_data {
                        let (left_parenthese, right_parenthese) =
                            self.extract_contained_span_tokens(parentheses)?;
                        parenthese_type.set_tokens(ParentheseTypeTokens {
                            left_parenthese,
                            right_parenthese,
                        });
                    }

                    self.types.push(parenthese_type.into());
                }
                ConvertWork::MakeTypePack { types, parentheses } => {
                    use ast::types::TypeInfo;

                    let mut type_pack = match types.iter().last() {
                        Some(TypeInfo::Variadic { .. }) | Some(TypeInfo::VariadicPack { .. }) => {
                            TypePack::from_iter(self.pop_types(types.len().saturating_sub(1))?)
                                .with_variadic_type(self.pop_variadic_type_pack()?)
                        }
                        Some(TypeInfo::GenericPack { .. }) => {
                            TypePack::from_iter(self.pop_types(types.len().saturating_sub(1))?)
                                .with_variadic_type(self.pop_generic_type_pack()?)
                        }
                        Some(TypeInfo::Array { .. })
                        | Some(TypeInfo::Basic(_))
                        | Some(TypeInfo::String(_))
                        | Some(TypeInfo::Boolean(_))
                        | Some(TypeInfo::Callback { .. })
                        | Some(TypeInfo::Generic { .. })
                        | Some(TypeInfo::Intersection { .. })
                        | Some(TypeInfo::Module { .. })
                        | Some(TypeInfo::Optional { .. })
                        | Some(TypeInfo::Table { .. })
                        | Some(TypeInfo::Typeof { .. })
                        | Some(TypeInfo::Tuple { .. })
                        | Some(TypeInfo::Union { .. })
                        | Some(_)
                        | None => TypePack::from_iter(self.pop_types(types.len())?),
                    };

                    if self.hold_token_data {
                        let (left_parenthese, right_parenthese) =
                            self.extract_contained_span_tokens(parentheses)?;
                        let commas = self.extract_tokens_from_punctuation(types)?;
                        type_pack.set_tokens(TypePackTokens {
                            left_parenthese,
                            right_parenthese,
                            commas,
                        });
                    }

                    self.type_packs.push(type_pack);
                }
            }
        }

        let block = self.blocks.pop().expect("root block should be converted");

        Ok(block)
    }

    fn convert_generic_type_parameters(
        &mut self,
        generics: &ast::types::GenericDeclaration,
    ) -> Result<GenericParameters, ConvertError> {
        let mut type_variables = Vec::new();
        let mut generic_type_packs = Vec::new();
        for parameter in generics.generics() {
            match parameter.parameter() {
                ast::types::GenericParameterInfo::Name(name) => {
                    if !generic_type_packs.is_empty() {
                        return Err(ConvertError::GenericDeclaration {
                            generics: generics.to_string(),
                        });
                    }
                    type_variables.push(self.convert_token_to_identifier(name)?);
                }
                ast::types::GenericParameterInfo::Variadic { name, ellipse } => {
                    let mut generic_pack =
                        GenericTypePack::new(self.convert_token_to_identifier(name)?);

                    if self.hold_token_data {
                        generic_pack.set_token(self.convert_token(ellipse)?);
                    }

                    generic_type_packs.push(generic_pack);
                }
                _ => {
                    return Err(ConvertError::GenericDeclaration {
                        generics: generics.to_string(),
                    })
                }
            }
        }
        let mut type_variables_iter = type_variables.into_iter();
        let mut generic_type_packs_iter = generic_type_packs.into_iter();
        let mut generic_parameters = type_variables_iter
            .next()
            .map(GenericParameters::from_type_variable)
            .or_else(|| {
                generic_type_packs_iter
                    .next()
                    .map(GenericParameters::from_generic_type_pack)
            })
            .ok_or_else(|| ConvertError::GenericDeclaration {
                generics: generics.to_string(),
            })?;
        if self.hold_token_data {
            let (opening_list, closing_list) =
                self.extract_contained_span_tokens(generics.arrows())?;
            let commas = self.extract_tokens_from_punctuation(generics.generics())?;
            generic_parameters.set_tokens(GenericParametersTokens {
                opening_list,
                closing_list,
                commas,
            });
        }
        Ok(generic_parameters)
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
                self.push_function_body_work(function.body());
            }
            ast::Stmt::GenericFor(generic_for) => {
                self.work_stack.push(ConvertWork::MakeGenericForStatement {
                    statement: generic_for,
                });
                self.push_work(generic_for.block());
                for type_specifier in generic_for.type_specifiers().flatten() {
                    self.push_work(type_specifier.type_info());
                }
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
                for type_specifier in local_assign
                    .type_specifiers()
                    .filter_map(|type_specifier| type_specifier)
                {
                    self.push_work(type_specifier.type_info());
                }
                for expression in local_assign.expressions().iter() {
                    self.push_work(expression);
                }
            }
            ast::Stmt::LocalFunction(local_function) => {
                self.work_stack
                    .push(ConvertWork::MakeLocalFunctionStatement {
                        statement: local_function,
                    });
                self.push_function_body_work(local_function.body());
            }
            ast::Stmt::NumericFor(numeric_for) => {
                self.work_stack.push(ConvertWork::MakeNumericForStatement {
                    statement: numeric_for,
                });
                if let Some(type_info) = numeric_for.type_specifier() {
                    self.push_work(type_info.type_info());
                }
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
            ast::Stmt::ExportedTypeDeclaration(exported_type_declaration) => {
                let type_declaration = exported_type_declaration.type_declaration();

                self.convert_type_declaration(
                    type_declaration,
                    Some(exported_type_declaration.export_token()),
                );
            }
            ast::Stmt::TypeDeclaration(type_declaration) => {
                self.convert_type_declaration(type_declaration, None);
            }
            _ => {
                return Err(ConvertError::Statement {
                    statement: statement.to_string(),
                })
            }
        }
        Ok(())
    }

    fn convert_type_declaration(
        &mut self,
        type_declaration: &'a ast::types::TypeDeclaration,
        export_token: Option<&'a tokenizer::TokenReference>,
    ) {
        self.work_stack
            .push(ConvertWork::MakeTypeDeclarationStatement {
                type_declaration,
                export_token,
            });
        self.push_work(type_declaration.type_definition());

        if let Some(generics) = type_declaration.generics() {
            for parameter in generics.generics() {
                if let Some(default_type) = parameter.default_type() {
                    match default_type {
                        ast::types::TypeInfo::Tuple { parentheses, types } => {
                            self.push_type_pack_work(types, parentheses);
                        }
                        _ => {
                            self.push_work(default_type);
                        }
                    }
                }
            }
        }
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
                        let (opening_bracket, closing_bracket) =
                            self.extract_contained_span_tokens(brackets)?;
                        entry.set_tokens(TableIndexEntryTokens {
                            opening_bracket,
                            closing_bracket,
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
            let (opening_brace, closing_brace) =
                self.extract_contained_span_tokens(table.braces())?;
            expression.set_tokens(TableTokens {
                opening_brace,
                closing_brace,
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
                            let (opening_bracket, closing_bracket) =
                                self.extract_contained_span_tokens(brackets)?;
                            index.set_tokens(IndexExpressionTokens {
                                opening_bracket,
                                closing_bracket,
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
                type_assertion,
            } => {
                if let Some(type_assertion) = type_assertion {
                    self.work_stack
                        .push(ConvertWork::MakeTypeCast { type_assertion });
                    self.push_work(type_assertion.cast_to());
                }
                match value.as_ref() {
                    ast::Value::Function((token, body)) => {
                        self.work_stack
                            .push(ConvertWork::MakeFunctionExpression { body, token });

                        self.push_function_body_work(body);
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
                        let mut expression = NumberExpression::from_str(
                            &number.token().to_string(),
                        )
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
                            self.work_stack.push(ConvertWork::PushExpression(
                                Expression::Identifier(
                                    self.convert_token_to_identifier(token_ref)?,
                                ),
                            ));
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
        }
        Ok(())
    }

    fn push_function_body_work(&mut self, body: &'a ast::FunctionBody) {
        self.push_work(body.block());
        if let Some(return_type) = body.return_type() {
            self.push_function_return_type(return_type.type_info());
        }
        for type_specifier in body.type_specifiers().flatten() {
            self.push_work(type_specifier.type_info());
        }
    }

    fn push_function_return_type(&mut self, return_type: &'a ast::types::TypeInfo) {
        self.push_work(ConvertWork::MakeFunctionReturnType {
            type_info: return_type,
        });
        match return_type {
            ast::types::TypeInfo::Tuple { types, parentheses } => {
                self.push_type_pack_work(types, parentheses);
            }
            _ => {
                self.push_work(return_type);
            }
        };
    }

    fn push_type_pack_work(
        &mut self,
        types: &'a ast::punctuated::Punctuated<ast::types::TypeInfo>,
        parentheses: &'a ast::span::ContainedSpan,
    ) {
        self.work_stack
            .push(ConvertWork::MakeTypePack { types, parentheses });

        for r#type in types {
            self.push_work(r#type)
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_type_info(
        &mut self,
        type_info: &'a ast::types::TypeInfo,
    ) -> Result<(), ConvertError> {
        use ast::types::TypeInfo;

        match type_info {
            TypeInfo::Array { braces, type_info } => {
                self.work_stack.push(ConvertWork::MakeArrayType { braces });

                self.push_work(type_info.as_ref());
            }
            TypeInfo::Basic(token_ref) => {
                if let TokenType::Symbol { symbol } = token_ref.token_type() {
                    let token = if self.hold_token_data {
                        Some(self.convert_token(token_ref)?)
                    } else {
                        None
                    };
                    let new_type = match symbol {
                        Symbol::Nil => Type::Nil(token),
                        _ => {
                            return Err(ConvertError::TypeInfo {
                                type_info: type_info.to_string(),
                            })
                        }
                    };
                    self.work_stack.push(ConvertWork::PushType(new_type));
                } else {
                    self.work_stack.push(ConvertWork::PushType(
                        TypeName::new(self.convert_token_to_identifier(token_ref)?).into(),
                    ));
                }
            }
            TypeInfo::String(token) => {
                self.work_stack.push(ConvertWork::PushType(
                    self.convert_string_type(token)?.into(),
                ));
            }
            TypeInfo::Boolean(token_ref) => {
                if let TokenType::Symbol { symbol } = token_ref.token_type() {
                    let token = if self.hold_token_data {
                        Some(self.convert_token(token_ref)?)
                    } else {
                        None
                    };
                    let new_type = match symbol {
                        Symbol::True => Type::True(token),
                        Symbol::False => Type::False(token),
                        _ => {
                            return Err(ConvertError::TypeInfo {
                                type_info: type_info.to_string(),
                            })
                        }
                    };
                    self.work_stack.push(ConvertWork::PushType(new_type));
                } else {
                    return Err(ConvertError::TypeInfo {
                        type_info: type_info.to_string(),
                    });
                }
            }
            TypeInfo::Callback {
                generics,
                parentheses,
                arguments,
                arrow,
                return_type,
            } => {
                let mut push_right_expression = None;

                let override_return_type = match return_type.as_ref() {
                    TypeInfo::Intersection {
                        left,
                        right,
                        ampersand,
                    } if matches!(left.as_ref(), TypeInfo::GenericPack { .. }) => {
                        // if we get a generic pack here then we are
                        // not making a function type
                        self.work_stack.push(ConvertWork::MakeIntersectionType {
                            operator: ampersand,
                        });

                        push_right_expression = Some(right.as_ref());

                        left
                    }
                    TypeInfo::Union { left, right, pipe }
                        if matches!(left.as_ref(), TypeInfo::GenericPack { .. }) =>
                    {
                        // if we get a generic pack here then we are
                        // not making a function type
                        self.work_stack
                            .push(ConvertWork::MakeUnionType { operator: pipe });

                        push_right_expression = Some(right.as_ref());

                        left
                    }

                    _ => return_type,
                };

                self.work_stack.push(ConvertWork::MakeFunctionType {
                    generics,
                    parentheses,
                    arguments,
                    arrow,
                });

                self.push_function_return_type(override_return_type);

                let mut has_variadic_type = false;

                for argument in arguments {
                    let argument_type = argument.type_info();
                    match argument_type {
                        TypeInfo::GenericPack { .. }
                        | TypeInfo::Variadic { .. }
                        | TypeInfo::VariadicPack { .. } => {
                            if has_variadic_type {
                                return Err(ConvertError::TypeInfo {
                                    type_info: type_info.to_string(),
                                });
                            }
                            has_variadic_type = true;
                        }
                        TypeInfo::Array { .. }
                        | TypeInfo::Basic(_)
                        | TypeInfo::String(_)
                        | TypeInfo::Boolean(_)
                        | TypeInfo::Callback { .. }
                        | TypeInfo::Generic { .. }
                        | TypeInfo::Intersection { .. }
                        | TypeInfo::Module { .. }
                        | TypeInfo::Optional { .. }
                        | TypeInfo::Table { .. }
                        | TypeInfo::Typeof { .. }
                        | TypeInfo::Tuple { .. }
                        | TypeInfo::Union { .. }
                        | _ => {}
                    }
                    self.push_work(argument_type);
                }

                if let Some(right) = push_right_expression {
                    self.push_work(right);
                }
            }
            TypeInfo::Generic {
                base,
                arrows,
                generics,
            } => {
                self.push_generic_type_work(base, arrows, generics, None);
            }
            TypeInfo::GenericPack { name, ellipse } => {
                let mut generic_pack =
                    GenericTypePack::new(self.convert_token_to_identifier(name)?);

                if self.hold_token_data {
                    generic_pack.set_token(self.convert_token(ellipse)?);
                }

                self.generic_type_packs.push(generic_pack);
            }
            TypeInfo::Intersection {
                left,
                ampersand,
                right,
            } => {
                match left.as_ref() {
                    TypeInfo::Variadic { ellipse, type_info } => {
                        self.work_stack
                            .push(ConvertWork::MakeVariadicTypePack { ellipse });

                        self.work_stack.push(ConvertWork::MakeIntersectionType {
                            operator: ampersand,
                        });

                        self.push_work(type_info.as_ref());
                    }
                    _ => {
                        self.work_stack.push(ConvertWork::MakeIntersectionType {
                            operator: ampersand,
                        });

                        self.push_work(left.as_ref());
                    }
                }
                self.push_work(right.as_ref());
            }
            TypeInfo::Union { left, pipe, right } => {
                match left.as_ref() {
                    TypeInfo::Variadic { ellipse, type_info } => {
                        self.work_stack
                            .push(ConvertWork::MakeVariadicTypePack { ellipse });

                        self.work_stack
                            .push(ConvertWork::MakeUnionType { operator: pipe });

                        self.push_work(type_info.as_ref());
                    }
                    _ => {
                        self.work_stack
                            .push(ConvertWork::MakeUnionType { operator: pipe });

                        self.push_work(left.as_ref());
                    }
                }
                self.push_work(right.as_ref());
            }
            TypeInfo::Module {
                module,
                punctuation,
                type_info,
            } => match type_info.as_ref() {
                ast::types::IndexedTypeInfo::Basic(name) => {
                    let mut type_field = TypeField::new(
                        self.convert_token_to_identifier(module)?,
                        TypeName::new(self.convert_token_to_identifier(name)?),
                    );

                    if self.hold_token_data {
                        type_field.set_token(self.convert_token(punctuation)?);
                    }

                    self.work_stack
                        .push(ConvertWork::PushType(type_field.into()));
                }
                ast::types::IndexedTypeInfo::Generic {
                    base,
                    arrows,
                    generics,
                } => {
                    self.push_generic_type_work(
                        base,
                        arrows,
                        generics,
                        Some((module, punctuation)),
                    );
                }
                _ => {
                    return Err(ConvertError::TypeInfo {
                        type_info: type_info.to_string(),
                    });
                }
            },
            TypeInfo::Optional {
                base,
                question_mark,
            } => {
                self.work_stack
                    .push(ConvertWork::MakeOptionalType { question_mark });

                self.push_work(base.as_ref());
            }
            TypeInfo::Table { braces, fields } => {
                self.work_stack
                    .push(ConvertWork::MakeTableType { braces, fields });

                let mut has_indexer_type = false;

                for field in fields {
                    use ast::types::TypeFieldKey;

                    match field.key() {
                        TypeFieldKey::Name(_) => {}
                        TypeFieldKey::IndexSignature { inner, .. } => {
                            if has_indexer_type {
                                return Err(ConvertError::TypeInfo {
                                    type_info: type_info.to_string(),
                                });
                            }

                            has_indexer_type = true;
                            self.push_work(inner);
                        }
                        key => {
                            return Err(ConvertError::TableTypeProperty {
                                property: key.to_string(),
                            });
                        }
                    }

                    self.push_work(field.value());
                }
            }
            TypeInfo::Typeof {
                typeof_token,
                parentheses,
                inner,
            } => {
                self.work_stack.push(ConvertWork::MakeExpressionType {
                    typeof_token,
                    parentheses,
                });

                self.push_work(inner.as_ref());
            }
            TypeInfo::Tuple { types, parentheses } => {
                if types.len() == 1 {
                    self.work_stack
                        .push(ConvertWork::MakeParentheseType { parentheses });
                    self.push_work(
                        types
                            .iter()
                            .next()
                            .expect("types should contain exactly one type at this point"),
                    );
                } else {
                    todo!()
                }
            }
            TypeInfo::Variadic { ellipse, type_info } => {
                self.work_stack
                    .push(ConvertWork::MakeVariadicTypePack { ellipse });

                self.push_work(type_info.as_ref());
            }
            TypeInfo::VariadicPack { ellipse, name } => {
                let mut variadic_type_pack =
                    VariadicTypePack::new(TypeName::new(self.convert_token_to_identifier(name)?));
                if self.hold_token_data {
                    variadic_type_pack.set_token(self.convert_token(ellipse)?);
                }
                self.variadic_type_packs.push(variadic_type_pack);
            }
            _ => {
                return Err(ConvertError::TypeInfo {
                    type_info: type_info.to_string(),
                });
            }
        }

        Ok(())
    }

    fn push_generic_type_work(
        &mut self,
        base: &'a tokenizer::TokenReference,
        arrows: &'a ast::span::ContainedSpan,
        generics: &'a ast::punctuated::Punctuated<ast::types::TypeInfo>,
        module: Option<(&'a tokenizer::TokenReference, &'a tokenizer::TokenReference)>,
    ) {
        self.work_stack
            .push(ConvertWork::MakeGenericType { base, module });

        self.work_stack
            .push(ConvertWork::MakeTypeParameters { arrows, generics });

        for parameter_type in generics {
            match parameter_type {
                ast::types::TypeInfo::Tuple { parentheses, types } => {
                    self.push_type_pack_work(types, parentheses);
                }
                _ => {
                    self.push_work(parameter_type);
                }
            }
        }
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
    fn convert_typed_identifier(
        &mut self,
        identifier: &tokenizer::TokenReference,
        type_specifier: Option<&ast::types::TypeSpecifier>,
    ) -> Result<TypedIdentifier, ConvertError> {
        let identifier = self.convert_token_to_identifier(identifier)?;

        Ok(if let Some(type_specifier) = type_specifier {
            let mut typed_identifier = identifier.with_type(self.pop_type()?);
            if self.hold_token_data {
                typed_identifier.set_colon_token(self.convert_token(type_specifier.punctuation())?);
            }
            typed_identifier
        } else {
            identifier.into()
        })
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
    fn extract_contained_span_tokens(
        &self,
        contained_span: &ast::span::ContainedSpan,
    ) -> Result<(Token, Token), ConvertError> {
        let (left, right) = contained_span.tokens();
        Ok((self.convert_token(left)?, self.convert_token(right)?))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_function_body_attributes(
        &mut self,
        body: &ast::FunctionBody,
        function_token: Token,
    ) -> Result<FunctionBuilder, ConvertError> {
        let mut builder = FunctionBuilder::from_block(self.pop_block()?);

        if let Some(return_type) = body.return_type() {
            if self.hold_token_data {
                builder.set_return_type_colon(self.convert_token(return_type.punctuation())?);
            }
            builder.set_return_type(self.pop_function_return_type()?);
        };

        for (param, type_specifier) in body.parameters().iter().zip(body.type_specifiers()) {
            match param {
                ast::Parameter::Ellipse(token) => {
                    if builder.is_variadic() {
                        return Err(ConvertError::FunctionParameters {
                            parameters: body.parameters().to_string(),
                        });
                    } else {
                        if let Some(type_specifier) = type_specifier {
                            builder.set_variadic_type(self.pop_type()?);

                            if self.hold_token_data {
                                builder.set_variable_arguments_colon(
                                    self.convert_token(type_specifier.punctuation())?,
                                );
                            }
                        } else {
                            builder.set_variadic();
                        }
                        if self.hold_token_data {
                            builder.set_variable_arguments_token(self.convert_token(token)?);
                        }
                    }
                }
                ast::Parameter::Name(name) => {
                    if builder.is_variadic() {
                        return Err(ConvertError::FunctionParameters {
                            parameters: body.parameters().to_string(),
                        });
                    }
                    let mut identifier = Identifier::new(name.token().to_string());
                    if self.hold_token_data {
                        identifier.set_token(self.convert_token(name)?);
                    }

                    if let Some(type_specifier) = type_specifier {
                        let type_value = self.pop_type()?;
                        let mut typed_identifier =
                            TypedIdentifier::from(identifier).with_type(type_value);
                        if self.hold_token_data {
                            typed_identifier
                                .set_colon_token(self.convert_token(type_specifier.punctuation())?);
                        }
                        builder.push_parameter(typed_identifier);
                    } else {
                        builder.push_parameter(identifier.into());
                    }
                }
                _ => {
                    return Err(ConvertError::FunctionParameter {
                        parameter: param.to_string(),
                    })
                }
            }
        }

        if let Some(generics) = body.generics() {
            let generic_parameters = self.convert_generic_type_parameters(generics)?;
            builder.set_generic_parameters(generic_parameters);
        }

        if self.hold_token_data {
            let (open, close) =
                self.extract_contained_span_tokens(body.parameters_parentheses())?;

            builder.set_parentheses_tokens(open, close);
            builder.set_parameter_commas(self.extract_tokens_from_punctuation(body.parameters())?);
            builder.set_function_token(function_token);
            builder.set_end_token(self.convert_token(body.end_token())?);
        }

        Ok(builder)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_string_expression(
        &self,
        string: &tokenizer::TokenReference,
    ) -> Result<StringExpression, ConvertError> {
        let mut expression =
            StringExpression::new(&string.token().to_string()).ok_or_else(|| {
                ConvertError::String {
                    string: string.to_string(),
                }
            })?;

        if self.hold_token_data {
            expression.set_token(self.convert_token(string)?);
        }
        Ok(expression)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_string_type(
        &self,
        string: &tokenizer::TokenReference,
    ) -> Result<StringType, ConvertError> {
        let mut expression =
            StringType::new(&string.token().to_string()).ok_or_else(|| ConvertError::String {
                string: string.to_string(),
            })?;
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
}

#[derive(Debug)]
enum ConvertWork<'a> {
    Block(&'a ast::Block),
    Statement(&'a ast::Stmt),
    LastStatement(&'a ast::LastStmt),
    Expression(&'a ast::Expression),
    Prefix(&'a ast::Prefix),
    Arguments(&'a ast::FunctionArgs),
    TypeInfo(&'a ast::types::TypeInfo),
    PushExpression(Expression),
    PushVariable(Variable),
    PushType(Type),
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
    MakeTypeDeclarationStatement {
        type_declaration: &'a ast::types::TypeDeclaration,
        export_token: Option<&'a tokenizer::TokenReference>,
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
    MakeFunctionReturnType {
        type_info: &'a ast::types::TypeInfo,
    },
    MakeVariadicTypePack {
        ellipse: &'a tokenizer::TokenReference,
    },
    MakeArrayType {
        braces: &'a ast::span::ContainedSpan,
    },
    MakeOptionalType {
        question_mark: &'a tokenizer::TokenReference,
    },
    MakeUnionType {
        operator: &'a tokenizer::TokenReference,
    },
    MakeIntersectionType {
        operator: &'a tokenizer::TokenReference,
    },
    MakeTableType {
        braces: &'a ast::span::ContainedSpan,
        fields: &'a ast::punctuated::Punctuated<ast::types::TypeField>,
    },
    MakeExpressionType {
        typeof_token: &'a tokenizer::TokenReference,
        parentheses: &'a ast::span::ContainedSpan,
    },
    MakeFunctionType {
        generics: &'a Option<ast::types::GenericDeclaration>,
        parentheses: &'a ast::span::ContainedSpan,
        arguments: &'a ast::punctuated::Punctuated<ast::types::TypeArgument>,
        arrow: &'a tokenizer::TokenReference,
    },
    MakeGenericType {
        base: &'a tokenizer::TokenReference,
        module: Option<(&'a tokenizer::TokenReference, &'a tokenizer::TokenReference)>,
    },
    MakeTypeParameters {
        arrows: &'a ast::span::ContainedSpan,
        generics: &'a ast::punctuated::Punctuated<ast::types::TypeInfo>,
    },
    MakeTypeCast {
        type_assertion: &'a ast::types::TypeAssertion,
    },
    MakeParentheseType {
        parentheses: &'a ast::span::ContainedSpan,
    },
    MakeTypePack {
        parentheses: &'a ast::span::ContainedSpan,
        types: &'a ast::punctuated::Punctuated<ast::types::TypeInfo>,
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

impl<'a> From<&'a ast::types::TypeInfo> for ConvertWork<'a> {
    fn from(type_info: &'a ast::types::TypeInfo) -> Self {
        ConvertWork::TypeInfo(type_info)
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
    String {
        string: String,
    },
    TypeInfo {
        type_info: String,
    },
    TableTypeProperty {
        property: String,
    },
    GenericDeclaration {
        generics: String,
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
            ConvertError::Expression { expression } => ("expression", expression),
            ConvertError::FunctionParameter { parameter } => ("parameter", parameter),
            ConvertError::FunctionParameters { parameters } => ("parameters", parameters),
            ConvertError::TableEntry { entry } => ("table entry", entry),
            ConvertError::BinaryOperator { operator } => ("binary operator", operator),
            ConvertError::CompoundOperator { operator } => ("compound operator", operator),
            ConvertError::UnaryOperator { operator } => ("unary operator", operator),
            ConvertError::String { string } => ("string", string),
            ConvertError::TypeInfo { type_info } => ("type", type_info),
            ConvertError::TableTypeProperty { property } => ("table type property", property),
            ConvertError::GenericDeclaration { generics } => ("generics", generics),
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
