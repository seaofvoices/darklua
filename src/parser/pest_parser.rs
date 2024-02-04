use pest::Parser;
use pest_derive::Parser;

use crate::{nodes::Block, parser::pest_converter::convert_block, utils::Timer, ParserError};

#[derive(Parser)]
#[grammar = "./parser/grammar.pest"]
struct LuauPestParser;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PestParser {
    hold_token_data: bool,
}

impl PestParser {
    pub fn parse(&self, code: &str) -> Result<Block, ParserError> {
        let pest_parse_timer = Timer::now();
        let parse_result = LuauPestParser::parse(Rule::module, code);
        log::trace!("pest parsing done in {}", pest_parse_timer.duration_label());

        let mut pairs = parse_result.map_err(ParserError::parsing2)?;

        let module = pairs.next().expect("expected module");

        log::trace!("start converting pest pairs");
        let conversion_timer = Timer::now();

        let block = match module.as_rule() {
            Rule::module => module
                .into_inner()
                .find_map(|pair| match pair.as_rule() {
                    Rule::block => Some(convert_block(pair).map_err(ParserError::converting)),
                    _ => None,
                })
                .expect("block expected inside module"),
            _ => unreachable!(),
        };

        log::trace!(
            " ⨽ completed pairs conversion in {}",
            conversion_timer.duration_label()
        );

        block
    }

    pub fn set_preserve_tokens(&mut self) {
        self.hold_token_data = true;
    }
}
