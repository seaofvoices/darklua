use pest::iterators::Pair;

use crate::{
    nodes::*,
    parser::{
        ast_converter::ConvertError,
        converter::{ConvertWork, Convertable, WorkScheduler},
        pest_converter::ExpressionConverter,
        pest_parser::Rule,
    },
};

use super::{
    convert_expression::convert_string_expression, convert_prefix::PrefixConverter, filter_tagged,
    find_first_tagged, get_first_tagged, pratt_parser::PrattParsing, submit_binding_list,
    submit_expression_list, submit_table_entry_list,
};

#[derive(Debug)]
pub(crate) struct ConvertPest<'a> {
    pair: Pair<'a, Rule>,
    kind: ConvertKind,
}

impl<'a> ConvertPest<'a> {
    pub(crate) fn block(pair: Pair<'a, Rule>) -> Self {
        Self {
            pair,
            kind: ConvertKind::Block,
        }
    }

    fn convert_statement<W: WorkScheduler<Convert = Self>>(
        self,
        stack: &mut W,
    ) -> Result<(), ConvertError> {
        match self.pair.as_rule() {
            Rule::call => {
                stack.push2(ConvertWork::MakeFunctionCallStatement);
                let mut scoped = W::new();
                PrefixConverter::new(&mut scoped).pratt_parse(self.pair.into_inner())?;
                stack.defer_merge_reverse(scoped);
            }
            Rule::assign_statement => {}
            Rule::compound_assign_statement => {
                let pairs = self.pair.into_inner();
                let variable_pair = get_first_tagged(pairs.clone(), "variable")?;
                let expr_pair = get_first_tagged(pairs.clone(), "expr")?;

                let operator = match get_first_tagged(pairs.clone(), "operator")?.as_str() {
                    "+=" => CompoundOperator::Plus,
                    "-=" => CompoundOperator::Minus,
                    "*=" => CompoundOperator::Asterisk,
                    "/=" => CompoundOperator::Slash,
                    "//=" => CompoundOperator::DoubleSlash,
                    "%=" => CompoundOperator::Percent,
                    "^=" => CompoundOperator::Caret,
                    "..=" => CompoundOperator::Concat,
                    _ => unreachable!(),
                };

                stack.push2(ConvertWork::MakeCompoundAssignStatement {
                    operator,
                    tokens: None,
                });

                stack.push2(ConvertWork::MakeVariable);

                let mut scoped = W::new();
                PrefixConverter::new(&mut scoped).pratt_parse(variable_pair.into_inner())?;
                stack.defer_merge_reverse(scoped);

                stack.push2(ConvertKind::Expression.as_work(expr_pair));
            }
            Rule::local_assign_statement => {
                let pairs = self.pair.into_inner();

                let identifier_count = submit_binding_list(pairs.clone(), stack);
                let expression_count = submit_expression_list(pairs, stack);

                stack.push_and_flush(ConvertWork::MakeLocalAssignStatement {
                    identifier_count,
                    expression_count,
                    tokens: None,
                });
            }
            Rule::do_statement => {
                let pairs = self.pair.into_inner();
                let block_pair = get_first_tagged(pairs, "block")?;

                stack.push2(ConvertWork::MakeDoStatement { tokens: None });
                stack.push2(ConvertKind::Block.as_work(block_pair));
            }
            Rule::while_statement => {
                let pairs = self.pair.into_inner();
                let block_pair = get_first_tagged(pairs.clone(), "block")?;
                let condition_pair = get_first_tagged(pairs, "condition")?;

                stack.push2(ConvertWork::MakeWhileStatement { tokens: None });
                stack.push2(ConvertKind::Block.as_work(block_pair));
                stack.push2(ConvertKind::Expression.as_work(condition_pair));
            }
            Rule::repeat_statement => {
                let pairs = self.pair.into_inner();
                let block_pair = get_first_tagged(pairs.clone(), "block")?;
                let condition_pair = get_first_tagged(pairs, "condition")?;

                stack.push2(ConvertWork::MakeRepeatStatement { tokens: None });
                stack.push2(ConvertKind::Block.as_work(block_pair));
                stack.push2(ConvertKind::Expression.as_work(condition_pair));
            }
            Rule::numeric_for_statement => {
                let pairs = self.pair.into_inner();
                stack.defer(
                    ConvertKind::TypedIdentifier.as_work(
                        pairs
                            .find_first_tagged("binding")
                            .expect("todo: binding expected"),
                    ),
                );
                stack.defer(ConvertKind::Block.as_work(get_first_tagged(pairs.clone(), "block")?));
                stack.defer(
                    ConvertKind::Expression.as_work(get_first_tagged(pairs.clone(), "start")?),
                );
                stack.defer(
                    ConvertKind::Expression.as_work(get_first_tagged(pairs.clone(), "end")?),
                );
                let has_step_expression = if let Some(step) = find_first_tagged(pairs, "step") {
                    stack.defer(ConvertKind::Expression.as_work(step));
                    true
                } else {
                    false
                };

                stack.push_and_flush(ConvertWork::MakeNumericForStatement {
                    has_step_expression,
                    tokens: None,
                });
            }
            Rule::generic_for_statement => {
                let pairs = self.pair.into_inner();

                stack.defer(ConvertKind::Block.as_work(get_first_tagged(pairs.clone(), "block")?));

                let identifier_count = submit_binding_list(pairs.clone(), stack);
                let expression_count = submit_expression_list(pairs, stack);

                stack.push_and_flush(ConvertWork::MakeGenericForStatement {
                    identifier_count,
                    expression_count,
                    tokens: None,
                });
            }
            Rule::if_statement => {
                let pairs = self.pair.into_inner();

                stack.defer(
                    ConvertKind::Expression.as_work(get_first_tagged(pairs.clone(), "condition")?),
                );

                stack.defer(ConvertKind::Block.as_work(get_first_tagged(pairs.clone(), "block")?));

                let has_else_block =
                    if let Some(else_block) = find_first_tagged(pairs.clone(), "else_block") {
                        stack.defer(ConvertKind::Block.as_work(else_block));
                        true
                    } else {
                        false
                    };

                let mut elseif_tokens = Vec::new();

                for branch in filter_tagged(pairs, "branch") {
                    let branch_pairs = branch.into_inner();
                    stack.defer(
                        ConvertKind::Expression
                            .as_work(get_first_tagged(branch_pairs.clone(), "condition")?),
                    );

                    stack.defer(
                        ConvertKind::Block.as_work(get_first_tagged(branch_pairs, "block")?),
                    );

                    elseif_tokens.push(None);
                }

                stack.push_and_flush(ConvertWork::MakeIfStatement {
                    elseif_tokens,
                    has_else_block,
                    tokens: None,
                });
            }
            Rule::local_function_statement => {
                let pairs = self.pair.into_inner();

                let identifier =
                    Identifier::new(get_first_tagged(pairs.clone(), "identifier")?.as_str());

                stack.defer(ConvertKind::Block.as_work(get_first_tagged(pairs.clone(), "block")?));

                let parameter_count = submit_binding_list(pairs.clone(), stack);
                let is_variadic =
                    if let Some(var_args) = find_first_tagged(pairs.clone(), "varargs") {
                        if let Some(type_pair) = find_first_tagged(var_args.into_inner(), "type") {
                            stack.defer(ConvertKind::Type.as_work(type_pair));
                        }
                        true
                    } else {
                        false
                    };

                stack.push_and_flush(ConvertWork::MakeLocalFunctionStatement {
                    identifier,
                    parameter_count,
                    is_variadic,
                    tokens: None,
                });
            }
            Rule::function_statement => {
                let pairs = self.pair.into_inner();

                let name_pairs = get_first_tagged(pairs.clone(), "name")?.into_inner();

                let function_name = {
                    let method =
                        find_first_tagged(name_pairs.clone(), "method").map(map_pair_as_identifier);
                    let mut iter_identifiers = filter_tagged(name_pairs, "identifier");
                    let first = iter_identifiers
                        .next()
                        .ok_or_else(|| todo!("todo add real error"))?;
                    let fields = iter_identifiers.map(map_pair_as_identifier).collect();

                    FunctionName::new(map_pair_as_identifier(first), fields, method)
                };

                stack.defer(ConvertKind::Block.as_work(get_first_tagged(pairs.clone(), "block")?));

                let parameter_count = submit_binding_list(pairs.clone(), stack);
                let is_variadic =
                    if let Some(var_args) = find_first_tagged(pairs.clone(), "varargs") {
                        if let Some(type_pair) = find_first_tagged(var_args.into_inner(), "type") {
                            stack.defer(ConvertKind::Type.as_work(type_pair));
                        }
                        true
                    } else {
                        false
                    };

                stack.push_and_flush(ConvertWork::MakeFunctionStatement {
                    function_name,
                    parameter_count,
                    is_variadic,
                    tokens: None,
                });
            }
            Rule::type_declaration => todo!(),
            _ => unreachable!(
                "todo: convert statement from `{:?}` > {:#?}",
                self.pair.as_rule(),
                self.pair
            ),
        }
        Ok(())
    }

    fn convert_block(
        self,
        stack: &mut impl WorkScheduler<Convert = Self>,
    ) -> Result<(), ConvertError> {
        let mut statement_count = 0;
        let mut has_last_statement = false;

        for pair in self.pair.into_inner() {
            if matches!(
                pair.as_rule(),
                Rule::return_statement | Rule::break_token | Rule::continue_token
            ) {
                has_last_statement = true;
                stack.defer(ConvertKind::LastStatement.as_work(pair).into());
            } else {
                statement_count += 1;
                stack.defer(ConvertKind::Statement.as_work(pair).into());
            }
        }

        stack.push_and_flush(ConvertWork::MakeBlock {
            statement_count,
            has_last_statement,
            tokens: None,
        });
        Ok(())
    }

    fn convert_last_statement(
        self,
        stack: &mut impl WorkScheduler<Convert = Self>,
    ) -> Result<(), ConvertError> {
        match self.pair.as_rule() {
            Rule::return_statement => {
                let expression_count = submit_expression_list(self.pair.into_inner(), stack);

                stack.push_and_flush(ConvertWork::MakeReturn {
                    expression_count,
                    tokens: None,
                });
            }
            Rule::break_token => {
                stack.push2(ConvertWork::PushLastStatement(LastStatement::new_break()));
            }
            Rule::continue_token => {
                stack.push2(ConvertWork::PushLastStatement(LastStatement::new_continue()));
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn convert_expression<W: WorkScheduler<Convert = Self>>(
        self,
        stack: &mut W,
    ) -> Result<(), ConvertError> {
        let mut scoped = W::new();
        ExpressionConverter::new(&mut scoped).pratt_parse(self.pair.into_inner())?;
        stack.defer_merge_reverse(scoped);
        Ok(())
    }

    fn convert_table_entry<W: WorkScheduler<Convert = Self>>(
        self,
        stack: &mut W,
    ) -> Result<(), ConvertError> {
        match self.pair.as_rule() {
            Rule::expr => {
                stack.push2(ConvertWork::MakeValueTableEntry);
                stack.push2(ConvertKind::Expression.as_work(self.pair));
            }
            Rule::table_index_entry => {
                stack.push2(ConvertWork::MakeIndexTableEntry { tokens: None });
                let pairs = self.pair.into_inner();
                stack.push2(
                    ConvertKind::Expression.as_work(get_first_tagged(pairs.clone(), "key")?),
                );
                stack.push2(ConvertKind::Expression.as_work(get_first_tagged(pairs, "value")?));
            }
            Rule::table_field_entry => {
                let pairs = self.pair.into_inner();

                let identifier =
                    Identifier::new(get_first_tagged(pairs.clone(), "field")?.as_str());

                stack.push2(ConvertWork::MakeFieldTableEntry {
                    identifier,
                    token: None,
                });
                stack.push2(ConvertKind::Expression.as_work(get_first_tagged(pairs, "value")?));
            }
            _ => unreachable!(
                "todo: convert table entry from `{:?}` > {:#?}",
                self.pair.as_rule(),
                self.pair
            ),
        }
        Ok(())
    }

    fn convert_call_arguments(
        self,
        stack: &mut impl WorkScheduler<Convert = Self>,
    ) -> Result<(), ConvertError> {
        match self.pair.as_rule() {
            Rule::function_arguments => {
                let pairs = self.pair.into_inner();

                if let Some(table_args) = find_first_tagged(pairs.clone(), "table_args") {
                    let entry_count = submit_table_entry_list(table_args.into_inner(), stack);
                    stack.push_and_flush(ConvertWork::MakeArgumentsFromTableEntries {
                        entry_count,
                        tokens: None,
                    });
                } else if let Some(string_args) = find_first_tagged(pairs.clone(), "string_args") {
                    stack.push2(ConvertWork::PushArguments(Arguments::String(
                        convert_string_expression(string_args.as_str())?.into(),
                    )));
                } else {
                    let expression_count = submit_expression_list(pairs, stack);

                    stack.push_and_flush(ConvertWork::MakeArgumentsFromExpressions {
                        expression_count,
                        tokens: None,
                    });
                }
            }
            _ => unreachable!(
                "todo: convert function arguments from `{:?}` > {:#?}",
                self.pair.as_rule(),
                self.pair
            ),
        }
        Ok(())
    }

    fn convert_typed_identifier(
        self,
        stack: &mut impl WorkScheduler<Convert = Self>,
    ) -> Result<(), ConvertError> {
        match self.pair.as_rule() {
            Rule::binding => {
                let pairs = self.pair.into_inner();

                let identifier =
                    Identifier::new(get_first_tagged(pairs.clone(), "identifier")?.as_str());

                if let Some(type_pair) = find_first_tagged(pairs, "type") {
                    stack.push2(ConvertWork::MakeTypedIdentifier {
                        identifier,
                        token: None,
                    });
                    stack.push2(ConvertKind::Type.as_work(type_pair));
                } else {
                    stack.push2(ConvertWork::PushTypedIdentifier(identifier.into()));
                }
            }
            _ => unreachable!(
                "todo: convert typed identifier from `{:?}` > {:#?}",
                self.pair.as_rule(),
                self.pair
            ),
        }
        Ok(())
    }
}

fn map_pair_as_identifier(pair: Pair<'_, Rule>) -> Identifier {
    Identifier::from(pair.as_str())
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ConvertKind {
    Block,
    TypedIdentifier,
    Statement,
    LastStatement,
    Expression,
    Arguments,
    Type,
    TableEntry,
}

impl ConvertKind {
    pub(crate) fn as_work<'a>(self, pair: Pair<'a, Rule>) -> ConvertWork<ConvertPest<'a>> {
        ConvertWork::Convert(ConvertPest { pair, kind: self })
    }
}

impl<'a> Convertable for ConvertPest<'a> {
    type Convert = ConvertPest<'a>;

    fn convert(
        self,
        work: &mut impl WorkScheduler<Convert = Self::Convert>,
    ) -> Result<(), ConvertError> {
        match self.kind {
            ConvertKind::Block => self.convert_block(work),
            ConvertKind::TypedIdentifier => self.convert_typed_identifier(work),
            ConvertKind::Statement => self.convert_statement(work),
            ConvertKind::LastStatement => self.convert_last_statement(work),
            ConvertKind::Expression => self.convert_expression(work),
            ConvertKind::TableEntry => self.convert_table_entry(work),
            ConvertKind::Arguments => self.convert_call_arguments(work),
            ConvertKind::Type => todo!(),
        }
    }
}
