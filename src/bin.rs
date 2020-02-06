mod cli;

use cli::{Command, Darklua};
use structopt::StructOpt;

fn main() {
    let darklua = Darklua::from_args();
    let global_options = darklua.global_options;

    match darklua.command {
        Command::Minify(options) => cli::minify::run(&options, &global_options),
    };
}
