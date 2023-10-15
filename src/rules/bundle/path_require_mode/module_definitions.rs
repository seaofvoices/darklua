use std::path::{Path, PathBuf};

use crate::frontend::DarkluaResult;
use crate::nodes::{
    Arguments, AssignStatement, Block, DoStatement, Expression, FieldExpression, FunctionCall,
    FunctionReturnType, Identifier, LastStatement, LocalAssignStatement, Position, ReturnStatement,
    Statement, TableExpression, Token, Type,
};
use crate::process::utils::{generate_identifier, identifier_permutator, CharPermutator};
use crate::rules::{Context, FlawlessRule, ShiftTokenLine};
use crate::DarkluaError;

use super::RequiredResource;

#[derive(Debug)]
pub(crate) struct BuildModuleDefinitions {
    modules_identifier: String,
    module_definitions: Vec<(String, Block, PathBuf)>,
    module_name_permutator: CharPermutator,
}

impl BuildModuleDefinitions {
    pub(crate) fn new(modules_identifier: impl Into<String>) -> Self {
        Self {
            modules_identifier: modules_identifier.into(),
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

        let module_name = generate_identifier(&mut self.module_name_permutator);

        self.module_definitions
            .push((module_name.clone(), block, require_path.to_path_buf()));

        let token_for_trivia = match call.get_arguments() {
            Arguments::Tuple(tuple) => tuple.get_tokens().map(|tokens| &tokens.closing_parenthese),
            Arguments::String(string) => string.get_token(),
            Arguments::Table(table) => table.get_tokens().map(|tokens| &tokens.closing_brace),
        };

        let field = if let Some(token_for_trivia) = token_for_trivia {
            let mut field_token = Token::from_content(module_name.clone());
            for trivia in token_for_trivia.iter_trailing_trivia() {
                field_token.push_trailing_trivia(trivia.clone());
            }
            Identifier::new(module_name).with_token(field_token)
        } else {
            Identifier::new(module_name)
        };

        Ok(FieldExpression::new(Identifier::from(&self.modules_identifier), field).into())
    }

    pub(crate) fn apply(mut self, block: &mut Block, context: &Context) {
        if self.module_definitions.is_empty() {
            return;
        }

        let modules_identifier = Identifier::from(&self.modules_identifier);

        let mut shift_lines = 0;
        for (module_name, module_block, _module_path) in self.module_definitions.iter_mut() {
            let inserted_lines = total_lines(module_block);

            let return_statement = module_block
                .take_last_statement()
                .map(|last| {
                    if let LastStatement::Return(statement) = last {
                        statement
                    } else {
                        unreachable!("module last statement should be a return statement")
                    }
                })
                .expect("module should have a last statement");

            let modules_prefix =
                if let Some(return_line_number) = return_statement
                    .get_tokens()
                    .and_then(|return_tokens| return_tokens.r#return.get_line_number())
                {
                    modules_identifier.clone().with_token(Token::from_position(
                        Position::line_number(self.modules_identifier.clone(), return_line_number),
                    ))
                } else {
                    modules_identifier.clone()
                };

            let return_value = return_statement
                .into_iter_expressions()
                .next()
                .expect("return statement should have one expression");

            module_block.push_statement(AssignStatement::from_variable(
                FieldExpression::new(modules_prefix, module_name.clone()),
                return_value,
            ));

            ShiftTokenLine::new(shift_lines).flawless_process(module_block, context);

            shift_lines += inserted_lines;
        }

        ShiftTokenLine::new(shift_lines).flawless_process(block, context);

        for (_, module_block, _) in self.module_definitions.drain(..).rev() {
            block.insert_statement(0, DoStatement::new(module_block));
        }
        block.insert_statement(
            0,
            LocalAssignStatement::from_variable(self.modules_identifier)
                .with_value(TableExpression::default()),
        );
    }
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
        .get_last_statement()
        .and_then(last_last_statement_token)
        .or_else(|| {
            block
                .iter_statements()
                .last()
                .and_then(last_statement_token)
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
        Type::Intersection(intersection) => last_type_token(intersection.get_right()),
        Type::Union(union) => last_type_token(union.get_right()),
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
            function_definition("function fn()\nend\n --comment\n") => 3,
            generic_for("for k, v in pairs({}) do\nend\n --comment") => 3,
            numeric_for("for i = 1, 10 do\n-- comment\nend\n") => 4,
            repeat_statement("\nrepeat\n-- do\nuntil condition\n") => 5,
            while_statement("\nwhile condition do\n-- do\nend\n") => 5,
            break_statement("break\n") => 2,
            continue_statement("continue\n") => 2,
        );
    }
}
