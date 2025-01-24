mod convert_work;
mod node_stacks;

pub(crate) use convert_work::{ConvertWork, Convertable, WorkScheduler, WorkStack};
pub(crate) use node_stacks::{NodeStacks, PushNode};

use crate::nodes::*;

use super::ast_converter::ConvertError;

pub(crate) fn convert_block<'a, C: std::fmt::Debug + Convertable<Convert = C>>(
    initial_work: ConvertWork<C>,
) -> Result<Block, ConvertError> {
    let mut work_stack = WorkStack::new_with(initial_work);

    let mut stacks = NodeStacks::default();

    while let Some(work) = work_stack.pop() {
        println!("DO WORK: {:#?}", work);
        match work {
            ConvertWork::Convert(convertable) => {
                convertable.convert(&mut work_stack)?;
            }
            ConvertWork::PushStatement(statement) => {
                stacks.push(statement);
            }
            ConvertWork::PushLastStatement(statement) => {
                stacks.push(statement);
            }
            ConvertWork::PushExpression(expression) => {
                stacks.push(expression);
            }
            ConvertWork::PushPrefix(prefix) => {
                stacks.push(prefix);
            }
            ConvertWork::PushArguments(arguments) => {
                stacks.push(arguments);
            }
            ConvertWork::PushTypedIdentifier(typed_identifier) => {
                stacks.push(typed_identifier);
            }
            ConvertWork::PushVariable(variable) => {
                stacks.push(variable);
            }
            ConvertWork::PushType(r#type) => {
                stacks.push(r#type);
            }
            ConvertWork::PushInterpolationSegment(segment) => {
                stacks.push(segment);
            }
            ConvertWork::MakeBlock {
                statement_count,
                has_last_statement,
                tokens,
            } => {
                let mut new_block: Block = Block::new(
                    stacks.pop_statements(statement_count)?,
                    has_last_statement
                        .then(|| stacks.pop_last_statement())
                        .transpose()?,
                );

                if let Some(tokens) = tokens {
                    new_block.set_tokens(tokens);
                }

                stacks.push(new_block);
            }
            ConvertWork::MakeDoStatement { tokens } => {
                let mut do_statement = DoStatement::new(stacks.pop_block()?);
                if let Some(tokens) = tokens {
                    do_statement.set_tokens(tokens);
                }
                stacks.push(Statement::from(do_statement));
            }
            ConvertWork::MakeRepeatStatement { tokens } => {
                let mut repeat_statement =
                    RepeatStatement::new(stacks.pop_block()?, stacks.pop_expression()?);
                if let Some(tokens) = tokens {
                    repeat_statement.set_tokens(tokens);
                }
                stacks.push(Statement::from(repeat_statement));
            }
            ConvertWork::MakeWhileStatement { tokens } => {
                let block = stacks.pop_block()?;
                let mut while_statement = WhileStatement::new(block, stacks.pop_expression()?);
                if let Some(tokens) = tokens {
                    while_statement.set_tokens(tokens);
                }
                stacks.push(Statement::from(while_statement));
            }
            ConvertWork::MakeNumericForStatement {
                has_step_expression,
                tokens,
            } => {
                let typed_identifier = stacks.pop_typed_identifier()?;
                let block = stacks.pop_block()?;
                let start = stacks.pop_expression()?;
                let end = stacks.pop_expression()?;
                let step = has_step_expression
                    .then(|| stacks.pop_expression())
                    .transpose()?;

                let mut numeric_for =
                    NumericForStatement::new(typed_identifier, start, end, step, block);

                if let Some(tokens) = tokens {
                    numeric_for.set_tokens(tokens);
                }
                stacks.push(Statement::from(numeric_for));
            }
            ConvertWork::MakeGenericForStatement {
                identifier_count,
                expression_count,
                tokens,
            } => {
                let block = stacks.pop_block()?;
                let identifiers = stacks.pop_typed_identifiers(identifier_count)?;
                let mut generic_for = GenericForStatement::new(
                    identifiers,
                    stacks.pop_expressions(expression_count)?,
                    block,
                );
                if let Some(tokens) = tokens {
                    generic_for.set_tokens(tokens);
                }
                stacks.push(Statement::from(generic_for));
            }
            ConvertWork::MakeLocalFunctionStatement {
                identifier,
                parameter_count,
                is_variadic,
                tokens,
            } => {
                let mut builder = FunctionBuilder::from_block(stacks.pop_block()?);

                for _ in 0..parameter_count {
                    builder.push_parameter(stacks.pop_typed_identifier()?);
                }

                if is_variadic {
                    builder.set_variadic();
                }

                let mut function = builder.into_local_function_statement(identifier, None);

                if let Some(tokens) = tokens {
                    function.set_tokens(tokens);
                }

                stacks.push(Statement::from(function));
            }
            ConvertWork::MakeFunctionStatement {
                function_name,
                parameter_count,
                is_variadic,
                tokens,
            } => {
                let mut builder = FunctionBuilder::from_block(stacks.pop_block()?);

                for _ in 0..parameter_count {
                    builder.push_parameter(stacks.pop_typed_identifier()?);
                }

                if is_variadic {
                    builder.set_variadic();
                }

                let mut function = builder.into_function_statement(function_name);

                if let Some(tokens) = tokens {
                    function.set_tokens(tokens);
                }

                stacks.push(Statement::from(function));
            }
            ConvertWork::MakeLocalAssignStatement {
                identifier_count,
                expression_count,
                tokens,
            } => {
                let identifiers = stacks.pop_typed_identifiers(identifier_count)?;

                let mut local_assign = LocalAssignStatement::new(
                    identifiers,
                    stacks.pop_expressions(expression_count)?,
                );

                if let Some(tokens) = tokens {
                    local_assign.set_tokens(tokens);
                }
                stacks.push(Statement::from(local_assign));
            }
            ConvertWork::MakeAssignStatement {
                variable_count,
                expression_count,
                tokens,
            } => {
                let variables = stacks.pop_variables(variable_count)?;
                let values = stacks.pop_expressions(expression_count)?;
                let mut assignment = AssignStatement::new(variables, values);
                if let Some(tokens) = tokens {
                    assignment.set_tokens(tokens);
                }
                stacks.push(Statement::from(assignment));
            }
            ConvertWork::MakeIfStatement {
                elseif_tokens,
                has_else_block,
                tokens,
            } => {
                let condition = stacks.pop_expression()?;
                let block = stacks.pop_block()?;
                let mut if_statement = IfStatement::create(condition, block);

                for elseif_tokens in elseif_tokens {
                    let elseif_condition = stacks.pop_expression()?;
                    let elseif_block = stacks.pop_block()?;
                    let mut branch = IfBranch::new(elseif_condition, elseif_block);
                    if let Some(tokens) = elseif_tokens {
                        branch.set_tokens(tokens);
                    }
                    if_statement.push_branch(branch);
                }

                if has_else_block {
                    if_statement.set_else_block(stacks.pop_block()?);
                }

                if let Some(tokens) = tokens {
                    if_statement.set_tokens(tokens);
                }
                stacks.push(Statement::from(if_statement));
            }
            ConvertWork::MakeCompoundAssignStatement { operator, tokens } => {
                let variable = stacks.pop_variable()?;
                let value = stacks.pop_expression()?;
                let mut assignment = CompoundAssignStatement::new(operator, variable, value);
                if let Some(tokens) = tokens {
                    assignment.set_tokens(tokens);
                }
                stacks.push(Statement::from(assignment));
            }
            ConvertWork::MakeParentheseExpression { tokens } => {
                let mut expression = ParentheseExpression::new(stacks.pop_expression()?);
                if let Some(tokens) = tokens {
                    expression.set_tokens(tokens);
                }
                stacks.push(Expression::from(expression))
            }
            ConvertWork::MakeReturn {
                expression_count,
                tokens,
            } => {
                let mut return_statement =
                    ReturnStatement::new(stacks.pop_expressions(expression_count)?);
                if let Some(tokens) = tokens {
                    return_statement.set_tokens(tokens);
                }
                stacks.push(LastStatement::from(return_statement));
            }
            ConvertWork::MakeBinaryExpression { operator, token } => {
                let right = stacks.pop_expression()?;
                let left = stacks.pop_expression()?;
                let mut binary = BinaryExpression::new(operator, left, right);
                if let Some(token) = token {
                    binary.set_token(token);
                }
                stacks.push(Expression::from(binary));
            }
            ConvertWork::MakeUnaryExpression { operator, token } => {
                let mut unary = UnaryExpression::new(operator, stacks.pop_expression()?);
                if let Some(token) = token {
                    unary.set_token(token);
                }
                stacks.push(Expression::from(unary));
            }
            ConvertWork::MakeIfExpression {
                elseif_branch_count,
            } => todo!(),
            ConvertWork::MakeFunctionExpression {
                parameter_count,
                is_variadic,
                tokens,
            } => {
                let mut builder = FunctionBuilder::from_block(stacks.pop_block()?);

                for _ in 0..parameter_count {
                    builder.push_parameter(stacks.pop_typed_identifier()?);
                }

                if is_variadic {
                    builder.set_variadic();
                }

                let mut function = builder.into_function_expression();

                if let Some(tokens) = tokens {
                    function.set_tokens(tokens);
                }

                stacks.push(Expression::from(function));
            }
            ConvertWork::MakeFunctionCallExpression {} => todo!(),
            ConvertWork::MakeFunctionCallStatement => {
                let prefix = stacks.pop_prefix()?;

                match prefix {
                    Prefix::Call(call) => {
                        stacks.push(Statement::from(call));
                    }
                    _ => unreachable!(),
                }
            }
            ConvertWork::MakeTypeDeclarationStatement {} => todo!(),
            ConvertWork::MakePrefixFromExpression => {
                let prefix = Prefix::from(stacks.pop_expression()?);
                stacks.push(prefix);
            }
            ConvertWork::MakeArgumentsFromExpressions {
                expression_count,
                tokens,
            } => {
                let mut tuple = TupleArguments::new(stacks.pop_expressions(expression_count)?);
                if let Some(tokens) = tokens {
                    tuple.set_tokens(tokens);
                }
                stacks.push(Arguments::from(tuple));
            }
            ConvertWork::MakeArgumentsFromTableEntries {
                entry_count,
                tokens,
            } => {
                let mut table = TableExpression::new(stacks.pop_table_entries(entry_count)?);
                if let Some(tokens) = tokens {
                    table.set_tokens(tokens);
                }
                stacks.push(Arguments::from(table));
            }
            ConvertWork::MakeTableExpression {
                entry_count,
                tokens,
            } => {
                let mut table = TableExpression::new(stacks.pop_table_entries(entry_count)?);
                if let Some(tokens) = tokens {
                    table.set_tokens(tokens);
                }
                stacks.push(Expression::from(table));
            }
            ConvertWork::MakeFieldTableEntry { identifier, token } => {
                let mut entry = TableFieldEntry::new(identifier, stacks.pop_expression()?);
                if let Some(token) = token {
                    entry.set_token(token);
                }
                stacks.push(TableEntry::from(entry));
            }
            ConvertWork::MakeIndexTableEntry { tokens } => {
                let key = stacks.pop_expression()?;
                let value = stacks.pop_expression()?;
                let mut entry = TableIndexEntry::new(key, value);
                if let Some(tokens) = tokens {
                    entry.set_tokens(tokens);
                }
                stacks.push(TableEntry::from(entry));
            }
            ConvertWork::MakeValueTableEntry => {
                let entry = stacks.pop_expression()?;
                stacks.push(TableEntry::Value(entry));
            }
            ConvertWork::MakeVariable => {
                let variable = match stacks.pop_prefix()? {
                    Prefix::Field(field) => Variable::from(*field),
                    Prefix::Identifier(identifier) => Variable::from(identifier),
                    Prefix::Index(index) => Variable::from(*index),
                    _ => todo!("todo create error"),
                };

                stacks.push(variable);
            }
            ConvertWork::MakePrefixExpression => {
                let prefix = stacks.pop_prefix()?;
                stacks.push(Expression::from(prefix));
            }
            ConvertWork::MakeIndexPrefix { tokens } => {
                let mut index =
                    IndexExpression::new(stacks.pop_prefix()?, stacks.pop_expression()?);
                if let Some(tokens) = tokens {
                    index.set_tokens(tokens);
                }
                stacks.push(Prefix::from(index));
            }
            ConvertWork::MakeFieldPrefix { identifier, token } => {
                let mut field = FieldExpression::new(stacks.pop_prefix()?, identifier);
                if let Some(token) = token {
                    field.set_token(token);
                }
                stacks.push(Prefix::from(field));
            }
            ConvertWork::MakeCallPrefix { method, tokens } => {
                let mut call =
                    FunctionCall::new(stacks.pop_prefix()?, stacks.pop_arguments()?, method);

                if let Some(tokens) = tokens {
                    call.set_tokens(tokens);
                }

                stacks.push(Prefix::from(call));
            }
            ConvertWork::MakeTypedIdentifier { identifier, token } => {
                let mut typed_identifier = identifier.with_type(stacks.pop_type()?);

                if let Some(token) = token {
                    typed_identifier.set_token(token);
                }

                stacks.push(typed_identifier);
            }
            ConvertWork::MakeInterpolatedString { segments } => {
                let interpolated_string =
                    InterpolatedStringExpression::new(stacks.pop_interpolation_segments(segments)?);

                stacks.push(Expression::from(interpolated_string));
            }
            ConvertWork::MakeInterpolationValueSegment => {
                let segment = ValueSegment::new(stacks.pop_expression()?);
                stacks.push(InterpolationSegment::Value(segment));
            }
            ConvertWork::MakeFunctionReturnType {} => todo!(),
            ConvertWork::MakeVariadicTypePack {} => todo!(),
            ConvertWork::MakeArrayType {} => todo!(),
            ConvertWork::MakeOptionalType {} => todo!(),
            ConvertWork::MakeUnionType {} => todo!(),
            ConvertWork::MakeIntersectionType {} => todo!(),
            ConvertWork::MakeTableType {} => todo!(),
            ConvertWork::MakeExpressionType {} => todo!(),
            ConvertWork::MakeFunctionType {} => todo!(),
            ConvertWork::MakeGenericType {} => todo!(),
            ConvertWork::MakeTypeParameters {} => todo!(),
            ConvertWork::MakeTypeCast {} => todo!(),
            ConvertWork::MakeParentheseType {} => todo!(),
            ConvertWork::MakeTypePack {} => todo!(),
        }
    }
    let mut block = stacks.pop_block().expect("root block should be converted");

    if let Some(tokens) = block.mutate_tokens() {
        // let token = self.convert_token(ast.eof())?;
        // if token.has_trivia() {
        //     tokens.final_token = Some(token);
        // }
    }

    Ok(block)
}
