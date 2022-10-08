pub mod error;
pub mod minify;
pub mod process;
pub mod utils;

use log::LevelFilter;
use structopt::StructOpt;

use self::error::CliError;

type CommandResult = Result<(), CliError>;

#[derive(Debug, StructOpt)]
pub struct GlobalOptions {
    /// Sets verbosity level (can be specified multiple times)
    #[structopt(long, short, global(true), parse(from_occurrences))]
    verbose: u8,
}

impl GlobalOptions {
    pub fn get_log_level_filter(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
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

impl Command {
    pub fn run(&self, global_options: &GlobalOptions) -> CommandResult {
        match self {
            Command::Minify(options) => minify::run(options, global_options),
            Command::Process(options) => process::run(options, global_options),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "darklua", about)]
pub struct Darklua {
    #[structopt(flatten)]
    global_options: GlobalOptions,
    /// The command to run. For specific help about each command, run `darklua <command> --help`
    #[structopt(subcommand)]
    command: Command,
}

impl Darklua {
    pub fn run(&self) -> CommandResult {
        self.command.run(&self.global_options)
    }

    pub fn get_log_level_filter(&self) -> LevelFilter {
        self.global_options.get_log_level_filter()
    }
}
