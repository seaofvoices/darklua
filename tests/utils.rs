use darklua_core::{Parser, ParsingError};
use darklua_core::nodes::Block;

#[allow(dead_code)]
pub fn parse_input(input: &str) -> Block {
    match Parser::default().parse(input) {
        Ok(block) => block,
        Err(error) => panic!("could not parse content: {:?}\ncontent:\n{}", error, input)
    }
}

#[allow(dead_code)]
pub fn try_parse_input(input: &str) -> Result<Block, ParsingError> {
    Parser::default().parse(input)
}
