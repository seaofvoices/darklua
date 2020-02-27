use crate::cli::GlobalOptions;
use crate::cli::error::CliError;
use crate::cli::utils::{
    maybe_plural,
    write_file,
    Config,
    FileProcessing,
};

use darklua_core::{LuaGenerator, ToLua, Parser};
use std::path::PathBuf;
use std::fs;
use std::process;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {
    /// Path to the lua file to minify.
    #[structopt(parse(from_os_str))]
    pub input_path: PathBuf,
    /// Where to output the result.
    #[structopt(parse(from_os_str))]
    pub output_path: PathBuf,
    /// Choose a specific configuration file.
    #[structopt(long, short)]
    pub config_path: Option<PathBuf>,
}

fn process(file: &FileProcessing, options: &Options, global: &GlobalOptions) -> Result<(), CliError> {
    let config = Config::new(&options.config_path, global)?;

    let source = &file.source;
    let output = &file.output;

    if !source.exists() {
        return Err(CliError::InputFileNotFound(source.clone()))
    }

    let input = fs::read_to_string(source)
        .map_err(|io_error| CliError::InputFile(format!("{}", io_error)))?;

    let parser = Parser::default();

    let mut block = parser.parse(&input)
        .map_err(|parser_error| CliError::Parser(source.clone(), parser_error))?;

    config.process.iter().for_each(|rule| rule.process(&mut block));

    let mut generator = LuaGenerator::new(config.column_span);
    block.to_lua(&mut generator);
    let minified = generator.into_string();

    write_file(&output, &minified)
        .map_err(|io_error| CliError::OutputFile(output.clone(), format!("{}", io_error)))?;

    if global.verbose > 0 {
        println!("Successfully processed <{}>", source.to_string_lossy());
    };

    Ok(())
}

pub fn run(options: &Options, global: &GlobalOptions) {
    let files = FileProcessing::find(&options.input_path, &options.output_path, global);

    let results: Vec<Result<(), CliError>> = files.iter()
        .map(|file_processing| process(file_processing, &options, global))
        .collect();

    let total_files = results.len();

    let errors: Vec<CliError> = results.into_iter()
        .filter_map(|result| match result {
            Ok(()) => None,
            Err(error) => Some(error),
        })
        .collect();

    let error_count = errors.len();

    if error_count == 0 {
        println!("Successfully processed {} file{}", total_files, maybe_plural(total_files));

    } else {
        let success_count = total_files - error_count;

        if success_count > 0 {
            eprintln!("Successfully processed {} file{}.", success_count, maybe_plural(success_count));
        }

        eprintln!("But {} error{} happened:", error_count, maybe_plural(error_count));

        errors.iter().for_each(|error| eprintln!("-> {}", error));
        process::exit(1);
    }
}
