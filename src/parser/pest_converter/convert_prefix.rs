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
    convert_expression::push_parenthese_expression,
    convert_pest::{ConvertKind, ConvertPest},
    find_first_tagged, get_first_tagged,
    pratt_parser::{Op, PrattConfig, PrattContext},
};

pub(crate) struct PrefixConverter<'i, 'w, W>
where
    W: WorkScheduler<Convert = ConvertPest<'i>>,
{
    stack: &'w mut W,
}

impl<'i, 'w, W> PrefixConverter<'i, 'w, W>
where
    W: WorkScheduler<Convert = ConvertPest<'i>>,
{
    pub(crate) fn new(stack: &'w mut W) -> Self {
        Self { stack }
    }
}

impl<'pratt, 'i, 'w, W> PrattContext<'pratt, 'i, Rule, Result<(), ConvertError>>
    for PrefixConverter<'i, 'w, W>
where
    W: WorkScheduler<Convert = ConvertPest<'i>>,
{
    fn config(&self) -> &PrattConfig<Rule> {
        &PREFIX_CONFIG
    }

    // call().result

    fn map_primary(&mut self, primary: Pair<'i, Rule>) -> Result<(), ConvertError> {
        // println!("PREFIX PRIMARY {:?}", primary.as_rule());
        match primary.as_rule() {
            Rule::identifier => {
                self.stack.push2(ConvertWork::PushPrefix(
                    Identifier::new(primary.as_str()).into(),
                ));
            }
            Rule::parenthese_expr => {
                push_parenthese_expression(primary, self.stack)?;
                self.stack.push2(ConvertWork::MakePrefixFromExpression);
            }
            _ => unreachable!(
                "todo: convert prefix from `{:?}` > {:#?}",
                primary.as_rule(),
                primary
            ),
        }
        Ok(())
    }

    fn map_postfix(
        &mut self,
        rhs: Result<(), ConvertError>,
        op: Pair<'i, Rule>,
    ) -> Result<(), ConvertError> {
        // println!("PREFIX POSTFIX {:?}", op.as_rule());
        rhs?;
        match op.as_rule() {
            Rule::suffix_index => {
                let key_expression_pair = get_first_tagged(op.into_inner(), "index")?;

                self.stack
                    .push2(ConvertKind::Expression.as_work(key_expression_pair));
                self.stack
                    .push2(ConvertWork::MakeIndexPrefix { tokens: None });
            }
            Rule::suffix_field => {
                let identifier = get_first_tagged(op.into_inner(), "field")?;

                self.stack.push2(ConvertWork::MakeFieldPrefix {
                    identifier: Identifier::new(identifier.as_str()),
                    token: None,
                });
            }
            Rule::suffix_call => {
                let pairs = op.into_inner();

                let method = find_first_tagged(pairs.clone(), "method")
                    .map(|pair| Identifier::new(pair.as_str()));

                let arguments_pair = get_first_tagged(pairs, "arguments")?;

                self.stack
                    .push2(ConvertKind::Arguments.as_work(arguments_pair));

                self.stack.push2(ConvertWork::MakeCallPrefix {
                    method,
                    tokens: None,
                });
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

lazy_static::lazy_static! {
    static ref PREFIX_CONFIG: PrattConfig<Rule> = {
        use Rule::*;

        PrattConfig::new()
            .op(Op::postfix(suffix_index) | Op::postfix(suffix_field) | Op::postfix(suffix_call))
   };
}
