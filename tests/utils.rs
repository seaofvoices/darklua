use darklua_core::Parser;
use darklua_core::nodes::Block;

pub fn parse_input(input: &str) -> Block {
    Parser::default().parse(input)
        .expect("could not parse file content")
}
