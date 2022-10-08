use crate::cli::error::CliError;
use crate::cli::utils::{
    log_array, maybe_plural, write_file, Config, FileProcessing, DEFAULT_COLUMN_SPAN,
};
use crate::cli::{CommandResult, GlobalOptions};

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
    input_path: PathBuf,
    /// Where to output the result.
    #[structopt(parse(from_os_str))]
    output_path: PathBuf,
    /// DEPRECATED - Reads the column span value from the given configuration file.
    /// Instead use `--column-span <number>` to pass the value directly. darklua
    /// will stop reading configuration files for the minify command in a future version
    #[structopt(long, short)]
    config_path: Option<PathBuf>,
    /// The maximum number of characters that should be written on a line.
    #[structopt(long)]
    column_span: Option<usize>,
}

fn minify(file: &FileProcessing, column_span: usize) -> CommandResult {
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

    let mut generator = DenseLuaGenerator::new(column_span);
    generator.write_block(&block);
    let minified = generator.into_string();

    write_file(output, &minified)
        .map_err(|io_error| CliError::OutputFile(output.clone(), format!("{}", io_error)))?;

    log::debug!("Successfully processed <{}>", source.display());

    Ok(())
}

pub fn run(options: &Options, global: &GlobalOptions) -> CommandResult {
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

    if options.config_path.is_some() {
        log::warn!(concat!(
            "Providing a configuration file for the minify command is now deprecated. ",
            "\nThe only part that was used was for the column width. Instead, pass ",
            "`--column-span <number>` directly in the command line"
        ))
    }

    let config = match Config::new(&options.config_path) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    };

    let results: Vec<CommandResult> = files
        .iter()
        .map(|file_processing| {
            minify(
                file_processing,
                options
                    .column_span
                    .or(config.column_span)
                    .unwrap_or(DEFAULT_COLUMN_SPAN),
            )
        })
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
        println!(
            "Successfully minified {} file{}",
            total_files,
            maybe_plural(total_files)
        );

        Ok(())
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
