mod cli;

use cli::{Command, Darklua};
use structopt::StructOpt;

fn main() {
    let darklua = Darklua::from_args();
    let global_options = darklua.global_options;

    let filter = global_options.get_log_level_filter();
    pretty_env_logger::formatted_builder()
        .filter_module("darklua", filter)
        .init();

    match darklua.command {
        Command::Minify(options) => cli::minify::run(&options, &global_options),
        Command::Process(options) => cli::process::run(&options, &global_options),
    };
}
