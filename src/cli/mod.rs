pub mod error;
pub mod minify;
pub mod process;
pub mod utils;

use log::LevelFilter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct GlobalOptions {
    /// Sets verbosity level (can be specified multiple times)
    #[structopt(long, short, global(true), parse(from_occurrences))]
    verbose: u8,
}

impl GlobalOptions {
    pub fn get_log_level_filter(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Minify lua files
    Minify(minify::Options),
    /// Process lua files with rules
    Process(process::Options),
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
