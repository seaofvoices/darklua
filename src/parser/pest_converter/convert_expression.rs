use std::str::FromStr;

use pest::iterators::Pair;

use crate::{
    nodes::*,
    parser::{
        ast_converter::ConvertError,
        converter::{ConvertWork, WorkScheduler},
        pest_parser::Rule,
    },
};

use super::{
    find_first_tagged, get_first_tagged,
    pratt_parser::{Op, PrattConfig, PrattContext, PrattParsing},
    submit_binding_list, submit_list, submit_table_entry_list, ConvertKind, ConvertPest,
    PrefixConverter,
};

pub(crate) struct ExpressionConverter<'i, 'w, W>
where
    W: WorkScheduler<Convert = ConvertPest<'i>>,
{
    stack: &'w mut W,
}

impl<'i, 'w, W> ExpressionConverter<'i, 'w, W>
where
    W: WorkScheduler<Convert = ConvertPest<'i>>,
{
    pub(crate) fn new(stack: &'w mut W) -> Self {
        Self { stack }
    }
}

pub(crate) fn convert_string_expression(
    string_repr: &str,
) -> Result<StringExpression, ConvertError> {
    let expression = StringExpression::new(string_repr).map_err(|_err| ConvertError::String {
        string: string_repr.to_string(),
    })?;

    // todo: add token

    Ok(expression)
}

pub(crate) fn push_parenthese_expression<'i, W>(
    primary: Pair<'i, Rule>,
    stack: &mut W,
) -> Result<(), ConvertError>
where
    W: WorkScheduler<Convert = ConvertPest<'i>>,
{
    let inner_pair = get_first_tagged(primary.into_inner(), "expr")?;
    stack.push2(ConvertKind::Expression.as_work(inner_pair));
    stack.push2(ConvertWork::MakeParentheseExpression { tokens: None });
    Ok(())
}

impl<'pratt, 'i, 'w, W> PrattContext<'pratt, 'i, Rule, Result<(), ConvertError>>
    for ExpressionConverter<'i, 'w, W>
where
    W: WorkScheduler<Convert = ConvertPest<'i>>,
{
    fn config(&self) -> &PrattConfig<Rule> {
        &EXPR_CONFIG
    }

    fn map_primary(&mut self, primary: Pair<'i, Rule>) -> Result<(), ConvertError> {
        match primary.as_rule() {
            Rule::nil_token => {
                self.stack
                    .push2(ConvertWork::PushExpression(Expression::nil()));
            }
            Rule::true_token => {
                self.stack.push2(ConvertWork::PushExpression(true.into()));
            }
            Rule::false_token => {
                self.stack.push2(ConvertWork::PushExpression(false.into()));
            }
            Rule::number => {
                let string_repr = primary.as_str();
                let expression = NumberExpression::from_str(string_repr).map_err(|err| {
                    ConvertError::Number {
                        number: string_repr.to_string(),
                        parsing_error: err.to_string(),
                    }
                })?;

                self.stack
                    .push2(ConvertWork::PushExpression(expression.into()));
            }
            Rule::string => {
                let string_repr = primary.as_str();
                self.stack.push2(ConvertWork::PushExpression(
                    convert_string_expression(string_repr)?.into(),
                ));
            }
            Rule::interpolated_string => {
                // todo!()
                self.stack.push2(ConvertWork::MakeInterpolatedString {});
            }
            Rule::var_args_expr => {
                self.stack
                    .push2(ConvertWork::PushExpression(Expression::variable_arguments()));
            }
            Rule::parenthese_expr => {
                push_parenthese_expression(primary, self.stack)?;
            }
            Rule::function_expr => {
                let pairs = primary.into_inner();

                let parameter_count = submit_binding_list(pairs.clone().rev(), self.stack);
                let is_variadic =
                    if let Some(var_args) = find_first_tagged(pairs.clone(), "varargs") {
                        if let Some(type_pair) = find_first_tagged(var_args.into_inner(), "type") {
                            self.stack.push2(ConvertKind::Type.as_work(type_pair));
                        }
                        true
                    } else {
                        false
                    };

                self.stack
                    .push2(ConvertKind::Block.as_work(get_first_tagged(pairs, "block")?));

                self.stack.push2(ConvertWork::MakeFunctionExpression {
                    parameter_count,
                    is_variadic,
                    tokens: None,
                });
            }
            Rule::prefix_expr => {
                PrefixConverter::new(self.stack).pratt_parse(primary.into_inner())?;
                self.stack.push2(ConvertWork::MakePrefixExpression);
            }
            Rule::table_expr => {
                let entry_count = submit_table_entry_list(primary.into_inner().rev(), self.stack);
                self.stack.push2(ConvertWork::MakeTableExpression {
                    entry_count,
                    tokens: None,
                });
            }
            _ => unreachable!(
                "todo: convert expression from `{:?}` > {:#?}",
                primary.as_rule(),
                primary
            ),
        }

        Ok(())
    }

    fn map_prefix<'a>(
        &mut self,
        op: Pair<'a, Rule>,
        rhs: Result<(), ConvertError>,
    ) -> Result<(), ConvertError> {
        rhs?;
        let operator = match op.as_rule() {
            Rule::unary_operator_length => UnaryOperator::Length,
            Rule::unary_operator_negate => UnaryOperator::Minus,
            Rule::not_token => UnaryOperator::Not,
            _ => unreachable!(),
        };
        self.stack.push2(ConvertWork::MakeUnaryExpression {
            operator,
            token: None,
        });
        Ok(())
    }

    fn map_infix<'a>(
        &mut self,
        lhs: Result<(), ConvertError>,
        op: Pair<'a, Rule>,
        rhs: Result<(), ConvertError>,
    ) -> Result<(), ConvertError> {
        lhs?;
        rhs?;
        let operator = match op.as_rule() {
            Rule::binary_operator_and => BinaryOperator::And,
            Rule::binary_operator_or => BinaryOperator::Or,
            Rule::binary_operator_equal => BinaryOperator::Equal,
            Rule::binary_operator_not_equal => BinaryOperator::NotEqual,
            Rule::binary_operator_lower => BinaryOperator::LowerThan,
            Rule::binary_operator_lower_equal => BinaryOperator::LowerOrEqualThan,
            Rule::binary_operator_greater => BinaryOperator::GreaterThan,
            Rule::binary_operator_greater_equal => BinaryOperator::GreaterOrEqualThan,
            Rule::binary_operator_add => BinaryOperator::Plus,
            Rule::binary_operator_subtract => BinaryOperator::Minus,
            Rule::binary_operator_multiply => BinaryOperator::Asterisk,
            Rule::binary_operator_divide => BinaryOperator::Slash,
            Rule::binary_operator_floor_divide => BinaryOperator::DoubleSlash,
            Rule::binary_operator_modulo => BinaryOperator::Percent,
            Rule::binary_operator_exponent => BinaryOperator::Caret,
            Rule::binary_operator_concat => BinaryOperator::Concat,
            _ => unreachable!(),
        };
        self.stack.push2(ConvertWork::MakeBinaryExpression {
            operator,
            token: None,
        });

        Ok(())
    }
}

lazy_static::lazy_static! {
    static ref EXPR_CONFIG: PrattConfig<Rule> = {
        use super::pratt_parser::Assoc::*;
        use Rule::*;

        PrattConfig::new()
            .op(Op::infix(binary_operator_and, Left))
            .op(Op::infix(binary_operator_or, Left))
            .op(
                Op::infix(binary_operator_equal, Left)
                | Op::infix(binary_operator_not_equal, Left)
                | Op::infix(binary_operator_lower, Left)
                | Op::infix(binary_operator_lower_equal, Left)
                | Op::infix(binary_operator_greater, Left)
                | Op::infix(binary_operator_greater_equal, Left)
            )
            .op(Op::infix(binary_operator_concat, Right))
            .op(Op::infix(binary_operator_add, Left) | Op::infix(binary_operator_subtract, Left))
            .op(
                Op::infix(binary_operator_multiply, Left)
                | Op::infix(binary_operator_divide, Left)
                | Op::infix(binary_operator_floor_divide, Left)
                | Op::infix(binary_operator_modulo, Left)
            )
            .op(Op::infix(binary_operator_exponent, Right))
            .op(Op::prefix(unary_operator_length) | Op::prefix(unary_operator_negate) | Op::prefix(not_token))
    };
}
