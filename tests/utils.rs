use std::panic::Location;
use std::path::Path;
use std::time::{Duration, Instant};

use darklua_core::nodes::Block;
use darklua_core::{Parser, ParserError, Resources};
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
        .try_init()
        .ok();
}

#[track_caller]
#[allow(dead_code)]
pub fn snapshot_file_process_file_errors(
    resources: &Resources,
    file_name: &str,
    snapshot_name: &str,
) {
    let errors = darklua_core::process(resources, darklua_core::Options::new(file_name))
        .result()
        .unwrap_err();

    let error_display: Vec<_> = errors.into_iter().map(|err| err.to_string()).collect();

    let caller_path = Path::new(Location::caller().file());
    let snapshot_dir = Path::new("..")
        .join(caller_path.parent().unwrap())
        .join("snapshots");

    let mut settings = insta::Settings::clone_current();
    settings.add_filter("\\\\", "/");
    settings.set_omit_expression(true);
    settings.set_snapshot_path(snapshot_dir);
    settings.bind(|| {
        insta::assert_snapshot!(snapshot_name, error_display.join("\n"));
    });
}

#[allow(dead_code)]
pub fn run_for_minimum_time<F: Fn()>(duration: Duration, func: F) {
    let start = Instant::now();

    loop {
        func();

        if Instant::now().duration_since(start) > duration {
            break;
        }
    }
}

#[allow(unused_macros)]
macro_rules! memory_resources {
    ($($path:literal => $content:expr),+$(,)?) => ({
        let resources = Resources::from_memory();
        $(
            resources.write($path, $content).unwrap();
        )*
        resources
    });
}

#[allow(unused_imports)]
pub(crate) use memory_resources;
