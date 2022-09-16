use std::time::{Duration, Instant};

use darklua_core::nodes::Block;
use darklua_core::{Parser, ParserError};

#[allow(dead_code)]
pub fn parse_input(input: &str) -> Block {
    match Parser::default().parse(input) {
        Ok(block) => block,
        Err(error) => panic!("could not parse content: {:?}\ncontent:\n{}", error, input),
    }
}

#[allow(dead_code)]
pub fn try_parse_input(input: &str) -> Result<Block, ParserError> {
    Parser::default().parse(input)
}

#[allow(dead_code)]
pub fn run_for_minimum_time<F: Fn()>(func: F, duration: Duration) {
    let start = Instant::now();

    loop {
        func();

        if Instant::now().duration_since(start) > duration {
            break;
        }
    }
}
