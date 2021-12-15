use crate::cli::error::CliError;
use crate::cli::utils::{log_array, maybe_plural, write_file, Config, FileProcessing};
use crate::cli::GlobalOptions;

use darklua_core::{
    generator::{DenseLuaGenerator, LuaGenerator},
    Parser,
};
use std::fs;
use std::path::PathBuf;
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

type MinifyResult = Result<(), CliError>;

fn minify(file: &FileProcessing, config: &Config) -> MinifyResult {
    let source = &file.source;
    let output = &file.output;

    if !source.exists() {
        return Err(CliError::InputFileNotFound(source.clone()));
    }

    let input = fs::read_to_string(source)
        .map_err(|io_error| CliError::InputFile(format!("{}", io_error)))?;

    let parser = Parser::default();

    let block = parser
        .parse(&input)
        .map_err(|parser_error| CliError::Parser(source.clone(), parser_error))?;

    let mut generator = DenseLuaGenerator::new(config.column_span);
    generator.write_block(&block);
    let minified = generator.into_string();

    write_file(output, &minified)
        .map_err(|io_error| CliError::OutputFile(output.clone(), format!("{}", io_error)))?;

    log::debug!("Successfully processed <{}>", source.display());

    Ok(())
}

pub fn run(options: &Options, global: &GlobalOptions) {
    log::debug!("running `minify`: {:?}", options);

    let files = FileProcessing::find(&options.input_path, &options.output_path, global);

    log::trace!(
        "planned work: {}",
        log_array(files.iter().map(|file| if file.is_in_place() {
            format!("{}", file.source.display())
        } else {
            format!("{} -> {}", file.source.display(), file.output.display())
        }))
    );

    let config = match Config::new(&options.config_path) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{}", error);
            return;
        }
    };

    let results: Vec<MinifyResult> = files
        .iter()
        .map(|file_processing| minify(file_processing, &config))
        .collect();

    let total_files = results.len();

    let errors: Vec<CliError> = results
        .into_iter()
        .filter_map(|result| match result {
            Ok(()) => None,
            Err(error) => Some(error),
        })
        .collect();

    let error_count = errors.len();

    if error_count == 0 {
        log::info!(
            "Successfully minified {} file{}",
            total_files,
            maybe_plural(total_files)
        );
    } else {
        let success_count = total_files - error_count;

        if success_count > 0 {
            eprintln!(
                "Successfully minified {} file{}.",
                success_count,
                maybe_plural(success_count),
            );
            eprintln!(
                "But {} error{} happened:",
                error_count,
                maybe_plural(error_count)
            );
        } else {
            eprintln!(
                "{} error{} happened:",
                error_count,
                maybe_plural(error_count)
            );
        }

        errors.iter().for_each(|error| eprintln!("-> {}", error));
        process::exit(1);
    }
}
