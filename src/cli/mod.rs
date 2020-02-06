pub mod minify;
pub mod utils;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct GlobalOptions {
    /// Sets verbosity level (can be specified multiple times)
    #[structopt(long, short, global(true), parse(from_occurrences))]
    pub verbose: u8,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Minify lua files
    Minify(minify::Options),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "darklua", about, author)]
pub struct Darklua {
    #[structopt(flatten)]
    pub global_options: GlobalOptions,
    /// The command to run. For specific help about each command, run `darklua <command> --help`
    #[structopt(subcommand)]
    pub command: Command,
}
