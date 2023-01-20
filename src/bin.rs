mod cli;

use std::process;

use clap::Parser;
use cli::Darklua;
use env_logger::{
    fmt::{Color, Style, StyledValue},
    Builder,
};
use log::Level;

fn main() {
    let darklua = Darklua::parse();

    let filter = darklua.get_log_level_filter();

    formatted_logger().filter_module("darklua", filter).init();

    match darklua.run() {
        Ok(()) => {}
        Err(err) => {
            process::exit(err.exit_code());
        }
    }
}

fn formatted_logger() -> Builder {
    let mut builder = Builder::new();
    builder.format(|f, record| {
        use std::io::Write;

        let mut style = f.style();
        let level = colored_level(&mut style, record.level());

        writeln!(f, " {} > {}", level, record.args(),)
    });
    builder
}

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value("INFO"),
        Level::Warn => style.set_color(Color::Yellow).value("WARN"),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}
