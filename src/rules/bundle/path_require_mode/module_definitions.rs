use std::path::{Path, PathBuf};

use crate::frontend::DarkluaResult;
use crate::nodes::{
    Arguments, AssignStatement, Block, DoStatement, Expression, FieldExpression, FunctionCall,
    FunctionExpression, FunctionName, FunctionReturnType, FunctionStatement, Identifier,
    IfStatement, IndexExpression, LastStatement, LocalAssignStatement, Prefix, ReturnStatement,
    Statement, StringExpression, TableEntry, TableExpression, Token, TupleArguments,
    TupleArgumentsTokens, Type, UnaryExpression, UnaryOperator,
};
use crate::process::utils::{generate_identifier, identifier_permutator, CharPermutator};
use crate::rules::{Context, FlawlessRule, ShiftTokenLine};
use crate::DarkluaError;

use super::RequiredResource;

#[derive(Debug)]
pub(crate) struct BuildModuleDefinitions {
    modules_identifier: String,
    module_cache_field: &'static str,
    module_load_field: &'static str,
    module_definitions: Vec<(String, Block, PathBuf)>,
    module_name_permutator: CharPermutator,
}

impl BuildModuleDefinitions {
    pub(crate) fn new(modules_identifier: impl Into<String>) -> Self {
        Self {
            modules_identifier: modules_identifier.into(),
            module_cache_field: "cache",
            module_load_field: "load",
            module_definitions: Vec::new(),
            module_name_permutator: identifier_permutator(),
        }
    }

    pub(crate) fn build_module_from_resource(
        &mut self,
        required_resource: RequiredResource,
        require_path: &Path,
        call: &FunctionCall,
    ) -> DarkluaResult<Expression> {
        let block = match required_resource {
            RequiredResource::Block(block) => {
                if let Some(LastStatement::Return(return_statement)) = block.get_last_statement() {
                    if return_statement.len() != 1 {
                        return Err(DarkluaError::custom(format!(
                            "invalid Lua module at `{}`: module must return exactly one value",
                            require_path.display()
                        )));
                    }
                } else {
                    return Err(DarkluaError::custom(format!(
                        "invalid Lua module at `{}`: module must end with a return statement",
                        require_path.display()
                    )));
                };
                block
            }
            RequiredResource::Expression(expression) => {
                Block::default().with_last_statement(ReturnStatement::one(expression))
            }
        };

        let module_name = self.generate_module_name();

        self.module_definitions
            .push((module_name.clone(), block, require_path.to_path_buf()));

        let token_trivia_identifier = match call.get_prefix() {
            Prefix::Identifier(require_identifier) => require_identifier.get_token(),
            _ => None,
        };

        let load_field = if let Some(token_trivia_identifier) = token_trivia_identifier {
            let mut field_token = Token::from_content(self.module_load_field);
            for trivia in token_trivia_identifier.iter_trailing_trivia() {
                field_token.push_trailing_trivia(trivia.clone());
            }
            Identifier::new(self.module_load_field).with_token(field_token)
        } else {
            Identifier::new(self.module_load_field)
        };

        let arguments = match call.get_arguments() {
            Arguments::Tuple(original_args) => {
                if let Some(original_tokens) = original_args.get_tokens() {
                    TupleArguments::default().with_tokens(TupleArgumentsTokens {
                        opening_parenthese: transfer_trivia(
                            Token::from_content("("),
                            &original_tokens.opening_parenthese,
                        ),
                        closing_parenthese: transfer_trivia(
                            Token::from_content(")"),
                            &original_tokens.closing_parenthese,
                        ),
                        commas: Vec::new(),
                    })
                } else {
                    TupleArguments::default()
                }
            }
            Arguments::String(string_expression) => {
                if let Some(string_token) = string_expression.get_token() {
                    TupleArguments::default().with_tokens(TupleArgumentsTokens {
                        opening_parenthese: Token::from_content("("),
                        closing_parenthese: transfer_trivia(Token::from_content(")"), string_token),
                        commas: Vec::new(),
                    })
                } else {
                    TupleArguments::default()
                }
            }
            Arguments::Table(_) => TupleArguments::default(),
        };

        let new_require_call = FunctionCall::from_prefix(FieldExpression::new(
            Identifier::from(&self.modules_identifier),
            load_field,
        ))
        .with_arguments(arguments.with_argument(StringExpression::from_value(module_name)))
        .into();

        Ok(new_require_call)
    }

    fn generate_module_name(&mut self) -> String {
        loop {
            let name = generate_identifier(&mut self.module_name_permutator);

            if name != self.module_cache_field && name != self.module_load_field {
                break name;
            }
        }
    }

    pub(crate) fn apply(mut self, block: &mut Block, context: &Context) {
        if self.module_definitions.is_empty() {
            return;
        }

        let modules_identifier = Identifier::from(&self.modules_identifier);

        let mut shift_lines = 0;
        for (_module_name, module_block, _module_path) in self.module_definitions.iter_mut() {
            let inserted_lines = total_lines(module_block);

            ShiftTokenLine::new(shift_lines).flawless_process(module_block, context);

            shift_lines += inserted_lines;
        }

        ShiftTokenLine::new(shift_lines).flawless_process(block, context);

        let statements = self
            .module_definitions
            .drain(..)
            .map(|(module_name, module_block, _)| {
                let function_name =
                    FunctionName::from_name(modules_identifier.clone()).with_field(module_name);
                FunctionStatement::new(function_name, module_block, Vec::new(), false)
            })
            .map(Statement::from)
            .collect();
        block.insert_statement(0, DoStatement::new(Block::new(statements, None)));

        let modules_table = self.build_modules_table();
        block.insert_statement(
            0,
            AssignStatement::from_variable(modules_identifier, modules_table),
        );
        block.insert_statement(
            0,
            LocalAssignStatement::from_variable(self.modules_identifier),
        );
    }

    fn build_modules_table(&self) -> TableExpression {
        let module_content_entry = "c";
        let parameter_name = "m";
        let index_cache = IndexExpression::new(
            FieldExpression::new(
                Identifier::from(&self.modules_identifier),
                self.module_cache_field,
            ),
            Identifier::from(parameter_name),
        );
        let load_function = FunctionExpression::from_block(
            Block::default()
                .with_statement(IfStatement::create(
                    UnaryExpression::new(UnaryOperator::Not, index_cache.clone()),
                    AssignStatement::from_variable(
                        index_cache.clone(),
                        TableExpression::default().append_entry(
                            TableEntry::from_string_key_and_value(
                                module_content_entry,
                                FunctionCall::from_prefix(IndexExpression::new(
                                    Identifier::from(&self.modules_identifier),
                                    Identifier::from(parameter_name),
                                )),
                            ),
                        ),
                    ),
                ))
                .with_last_statement(ReturnStatement::one(FieldExpression::new(
                    index_cache,
                    module_content_entry,
                ))),
        )
        .with_parameter(parameter_name);

        TableExpression::default()
            .append_entry(TableEntry::from_string_key_and_value(
                self.module_cache_field,
                TableExpression::default(),
            ))
            .append_field(self.module_load_field, load_function)
    }
}

fn transfer_trivia(mut receiving_token: Token, take_token: &Token) -> Token {
    for (content, kind) in take_token.iter_trailing_trivia().filter_map(|trivia| {
        trivia
            .try_read()
            .map(str::to_owned)
            .zip(Some(trivia.kind()))
    }) {
        receiving_token.push_trailing_trivia(kind.with_content(content));
    }
    receiving_token
}

fn total_lines(block: &Block) -> usize {
    last_block_token(block)
        .and_then(|token| {
            token
                .iter_trailing_trivia()
                .last()
                .and_then(|trivia| {
                    trivia.get_line_number().map(|line| {
                        line + trivia
                            .try_read()
                            .unwrap_or_default()
                            .chars()
                            .filter(|c| *c == '\n')
                            .count()
                    })
                })
                .or_else(|| token.get_line_number())
        })
        .unwrap_or(0)
}

fn last_block_token(block: &Block) -> Option<&Token> {
    block
        .get_tokens()
        .and_then(|tokens| tokens.final_token.as_ref())
        .or_else(|| {
            block
                .get_last_statement()
                .and_then(last_last_statement_token)
                .or_else(|| {
                    block
                        .iter_statements()
                        .last()
                        .and_then(last_statement_token)
                })
        })
}

fn last_statement_token(statement: &Statement) -> Option<&Token> {
    match statement {
        Statement::Assign(assign) => assign.last_value().and_then(last_expression_token),
        Statement::Do(do_statement) => do_statement.get_tokens().map(|tokens| &tokens.end),
        Statement::Call(call) => last_call_token(call),
        Statement::CompoundAssign(assign) => last_expression_token(assign.get_value()),
        Statement::Function(function) => function.get_tokens().map(|tokens| &tokens.end),
        Statement::GenericFor(generic_for) => generic_for.get_tokens().map(|tokens| &tokens.end),
        Statement::If(if_statement) => if_statement.get_tokens().map(|tokens| &tokens.end),
        Statement::LocalAssign(local_assign) => local_assign
            .iter_values()
            .last()
            .and_then(last_expression_token)
            .or_else(|| {
                local_assign
                    .iter_variables()
                    .last()
                    .and_then(|identifier| identifier.get_token())
            }),
        Statement::LocalFunction(local_function) => {
            local_function.get_tokens().map(|tokens| &tokens.end)
        }
        Statement::NumericFor(numeric_for) => numeric_for.get_tokens().map(|tokens| &tokens.end),
        Statement::Repeat(repeat) => last_expression_token(repeat.get_condition()),
        Statement::While(while_statement) => while_statement.get_tokens().map(|tokens| &tokens.end),
        Statement::TypeDeclaration(type_declaration) => {
            last_type_token(type_declaration.get_type())
        }
    }
}

fn last_last_statement_token(last: &LastStatement) -> Option<&Token> {
    match last {
        LastStatement::Break(token) | LastStatement::Continue(token) => token.as_ref(),
        LastStatement::Return(return_statement) => return_statement
            .iter_expressions()
            .last()
            .and_then(last_expression_token)
            .or_else(|| return_statement.get_tokens().map(|tokens| &tokens.r#return)),
    }
}

fn last_expression_token(expression: &Expression) -> Option<&Token> {
    match expression {
        Expression::Binary(binary) => last_expression_token(binary.right()),
        Expression::Call(call) => last_call_token(call),
        Expression::Field(field) => field.get_field().get_token(),
        Expression::Function(function) => function.get_tokens().map(|tokens| &tokens.end),
        Expression::Identifier(identifier) => identifier.get_token(),
        Expression::If(if_expression) => last_expression_token(if_expression.get_else_result()),
        Expression::Index(index) => index.get_tokens().map(|tokens| &tokens.closing_bracket),
        Expression::Number(number) => number.get_token(),
        Expression::Parenthese(parentheses) => parentheses
            .get_tokens()
            .map(|tokens| &tokens.right_parenthese),
        Expression::String(string) => string.get_token(),
        Expression::InterpolatedString(string) => {
            string.get_tokens().map(|tokens| &tokens.closing_tick)
        }
        Expression::Table(table) => table.get_tokens().map(|tokens| &tokens.closing_brace),
        Expression::Nil(token)
        | Expression::False(token)
        | Expression::True(token)
        | Expression::VariableArguments(token) => token.as_ref(),
        Expression::Unary(unary) => last_expression_token(unary.get_expression()),
        Expression::TypeCast(type_cast) => last_type_token(type_cast.get_type()),
    }
}

fn last_type_token(r#type: &Type) -> Option<&Token> {
    match r#type {
        Type::Name(name) => {
            if let Some(type_params) = name.get_type_parameters() {
                type_params.get_tokens().map(|tokens| &tokens.closing_list)
            } else {
                name.get_type_name().get_token()
            }
        }
        Type::Field(field) => {
            if let Some(type_params) = field.get_type_name().get_type_parameters() {
                type_params.get_tokens().map(|tokens| &tokens.closing_list)
            } else {
                field.get_type_name().get_type_name().get_token()
            }
        }
        Type::True(token) | Type::False(token) | Type::Nil(token) => token.as_ref(),
        Type::String(string) => string.get_token(),
        Type::Array(array) => array.get_tokens().map(|tokens| &tokens.closing_brace),
        Type::Table(table) => table.get_tokens().map(|tokens| &tokens.closing_brace),
        Type::TypeOf(expression_type) => expression_type
            .get_tokens()
            .map(|tokens| &tokens.closing_parenthese),
        Type::Parenthese(parenthese) => parenthese
            .get_tokens()
            .map(|tokens| &tokens.right_parenthese),
        Type::Function(function) => match function.get_return_type() {
            FunctionReturnType::Type(return_type) => last_type_token(return_type),
            FunctionReturnType::TypePack(type_pack) => type_pack
                .get_tokens()
                .map(|tokens| &tokens.right_parenthese),
            FunctionReturnType::GenericTypePack(generic_pack) => generic_pack.get_token(),
            FunctionReturnType::VariadicTypePack(variadic_pack) => {
                last_type_token(variadic_pack.get_type())
            }
        },
        Type::Optional(optional) => optional.get_token(),
        Type::Intersection(intersection) => last_type_token(intersection.last_type()),
        Type::Union(union_type) => last_type_token(union_type.last_type()),
    }
}

fn last_call_token(call: &FunctionCall) -> Option<&Token> {
    match call.get_arguments() {
        Arguments::Tuple(tuple) => tuple.get_tokens().map(|tokens| &tokens.closing_parenthese),
        Arguments::String(string) => string.get_token(),
        Arguments::Table(table) => table.get_tokens().map(|tokens| &tokens.closing_brace),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod test_total_lines {
        use super::*;

        macro_rules! test_total_lines {
            (
                $($name:ident ($code:literal) => $value:expr),+,
            ) => {
                $(
                    #[test]
                    fn $name() {
                        let code = $code;

                        let mut block = $crate::Parser::default().preserve_tokens().parse(&code).unwrap();

                        let resources = $crate::Resources::from_memory();
                        let context = $crate::rules::ContextBuilder::new(
                                "placeholder",
                                &resources,
                                &code
                            )
                            .build();

                        $crate::rules::ReplaceReferencedTokens::default()
                        .flawless_process(&mut block, &context);

                        let received_lines = total_lines(&block);
                        assert_eq!(
                            received_lines,
                            $value,
                            "expected {} line{} but received {}.\n{:#?}",
                            $value,
                            if $value > 1 { "s" } else { "" },
                            received_lines,
                            block,
                        );
                    }
                )*
            };
        }

        test_total_lines!(
            return_statement("return\n") => 2,
            return_one("return 1") => 1,
            return_true("return true") => 1,
            return_false("return true,\n\tfalse\n") => 3,
            return_nil("return nil\n") => 2,
            return_string("return 'hello' --end\n") => 2,
            return_not_variable("return not variable") => 1,
            return_function_call("return call()") => 1,
            return_variadic_args("return ... -- comment") => 1,
            return_parenthese("return (\ncall()\n)") => 3,
            return_table("return {\n\t}\n") => 3,
            return_function_expression("return function(arg1, ...)\nend -- ") => 2,
            return_function_call_with_table_arguments("return call {\nelement\n}") => 3,
            function_call_with_table_arguments("call {\nelement\n}") => 3,
            require_with_string_argument("require 'module.lua'\n") => 2,
            return_require_with_string_argument("return require 'module.lua'\n") => 2,
            return_if_expression("return if condition then\n\tok\nelse\n\terr") => 4,
            if_statement("if condition then\n\treturn ok\nelse\n\treturn err\nend -- end if") => 5,
            do_statement("do\n--comment\n\n\nend") => 5,
            compound_assign("\nvar += 10.5") => 2,
            assign_with_binary_expression("var = var + 2") => 1,
            local_assign_with_field_expression("local var =\n\tobject.prop\n-- end") => 3,
            local_assign_with_index_expression("local var =\n\tobject['prop']\n-- end") => 3,
            local_function_definition("local function fn()\nend\n") => 3,
            function_definition("function fn()\nend\n --comment\n") => 4,
            generic_for("for k, v in pairs({}) do\nend\n --comment") => 3,
            numeric_for("for i = 1, 10 do\n-- comment\nend\n") => 4,
            repeat_statement("\nrepeat\n-- do\nuntil condition\n") => 5,
            while_statement("\nwhile condition do\n-- do\nend\n") => 5,
            break_statement("break\n") => 2,
            continue_statement("continue\n") => 2,
        );
    }
}
