mod convert_expression;
mod convert_pest;
mod convert_prefix;
mod pratt_parser;

use convert_expression::ExpressionConverter;
use convert_pest::{ConvertKind, ConvertPest};
use convert_prefix::PrefixConverter;
use pest::iterators::Pair;

use crate::nodes::Block;

use super::{
    ast_converter::ConvertError,
    converter::{self, WorkScheduler},
    pest_parser::Rule,
};

pub(crate) fn convert_block<'a>(pair: Pair<'a, Rule>) -> Result<Block, ConvertError> {
    converter::convert_block(ConvertPest::block(pair).into())
}

#[inline]
fn filter_tagged<'a>(
    pairs: impl Iterator<Item = Pair<'a, Rule>>,
    tag: &'static str,
) -> impl Iterator<Item = Pair<'a, Rule>> {
    pairs.filter(move |pair| pair.as_node_tag().filter(|t| *t == tag).is_some())
}

#[inline]
fn find_first_tagged<'a>(
    pairs: impl Iterator<Item = Pair<'a, Rule>>,
    tag: &'static str,
) -> Option<Pair<'a, Rule>> {
    for pair in pairs {
        if pair.as_node_tag().filter(|t| *t == tag).is_some() {
            return Some(pair);
        }
    }
    None
}

fn get_first_tagged<'a>(
    pairs: impl Iterator<Item = Pair<'a, Rule>>,
    tag: &'static str,
) -> Result<Pair<'a, Rule>, ConvertError> {
    find_first_tagged(pairs, tag)
        .ok_or_else(|| ConvertError::InternalMissingPestTag { tag_name: tag })
}

fn submit_binding_list<'a>(
    pairs: impl Iterator<Item = Pair<'a, Rule>>,
    stack: &mut impl WorkScheduler<Convert = ConvertPest<'a>>,
) -> usize {
    submit_list(pairs, "binding", ConvertKind::TypedIdentifier, stack)
}

fn submit_expression_list<'a>(
    pairs: impl Iterator<Item = Pair<'a, Rule>>,
    stack: &mut impl WorkScheduler<Convert = ConvertPest<'a>>,
) -> usize {
    submit_list(pairs, "expr", ConvertKind::Expression, stack)
}

fn submit_table_entry_list<'a>(
    pairs: impl Iterator<Item = Pair<'a, Rule>>,
    stack: &mut impl WorkScheduler<Convert = ConvertPest<'a>>,
) -> usize {
    submit_list(pairs, "table_entry", ConvertKind::TableEntry, stack)
}

fn submit_list<'a>(
    pairs: impl Iterator<Item = Pair<'a, Rule>>,
    tag: &'static str,
    kind: ConvertKind,
    stack: &mut impl WorkScheduler<Convert = ConvertPest<'a>>,
) -> usize {
    let mut count = 0;

    for pair in filter_tagged(pairs, tag) {
        count += 1;
        stack.defer(kind.as_work(pair));
    }

    count
}
