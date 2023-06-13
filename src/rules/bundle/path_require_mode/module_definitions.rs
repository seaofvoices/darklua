use std::path::{Path, PathBuf};

use crate::frontend::DarkluaResult;
use crate::nodes::{
    Arguments, AssignStatement, Block, DoStatement, Expression, FieldExpression, FunctionCall,
    Identifier, LastStatement, LocalAssignStatement, Position, ReturnStatement, Statement,
    TableExpression, Token,
};
use crate::process::utils::{generate_identifier, identifier_permutator, CharPermutator};
use crate::rules::{Context, FlawlessRule, ShiftTokenLine};
use crate::DarkluaError;

use super::RequiredResource;

#[derive(Debug)]
pub(crate) struct BuildModuleDefinitions<'a> {
    modules_identifier: &'a str,
    module_definitions: Vec<(String, Block, PathBuf)>,
    module_name_permutator: CharPermutator,
}

impl<'a> BuildModuleDefinitions<'a> {
    pub(crate) fn new(modules_identifier: &'a str) -> Self {
        Self {
            modules_identifier,
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

        Ok(FieldExpression::new(Identifier::from(self.modules_identifier), field).into())
    }

    pub(crate) fn apply(mut self, block: &mut Block, context: &Context) {
        if self.module_definitions.is_empty() {
            return;
        }

        let modules_identifier = Identifier::from(self.modules_identifier);

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

            let modules_prefix = if let Some(return_line_number) = return_statement
                .get_tokens()
                .and_then(|return_tokens| return_tokens.r#return.get_line_number())
            {
                modules_identifier
                    .clone()
                    .with_token(Token::from_position(Position::line_number(
                        self.modules_identifier.to_owned(),
                        return_line_number,
                    )))
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
        .and_then(last_last_statement_line)
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
        Statement::CompoundAssign(_) => todo!(),
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
    }
}

fn last_last_statement_line(last: &LastStatement) -> Option<&Token> {
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
        Expression::Field(field) => field.get_token(),
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
    }
}

fn last_call_token(call: &FunctionCall) -> Option<&Token> {
    match call.get_arguments() {
        Arguments::Tuple(tuple) => tuple.get_tokens().map(|tokens| &tokens.closing_parenthese),
        Arguments::String(string) => string.get_token(),
        Arguments::Table(table) => table.get_tokens().map(|tokens| &tokens.closing_brace),
    }
}
