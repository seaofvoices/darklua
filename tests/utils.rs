use std::time::{Duration, Instant};

use darklua_core::nodes::Block;
use darklua_core::{Parser, ParserError};
use env_logger::fmt::Color;
use log::Level;

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
pub fn setup_logger(level_filter: log::LevelFilter) {
    env_logger::Builder::new()
        .format(|f, record| {
            use std::io::Write;

            let mut style = f.style();
            let level = match record.level() {
                Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
                Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
                Level::Info => style.set_color(Color::Green).value("INFO"),
                Level::Warn => style.set_color(Color::Yellow).value("WARN"),
                Level::Error => style.set_color(Color::Red).value("ERROR"),
            };

            writeln!(f, " {} > {}", level, record.args(),)
        })
        .filter_module("darklua", level_filter)
        .init();
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
