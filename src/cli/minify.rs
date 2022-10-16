use crate::cli::error::CliError;
use crate::cli::utils::maybe_plural;
use crate::cli::{CommandResult, GlobalOptions};

use darklua_core::{Configuration, GeneratorParameters, Resources};
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {
    /// Path to the lua file to minify.
    #[structopt(parse(from_os_str))]
    input_path: PathBuf,
    /// Where to output the result.
    #[structopt(parse(from_os_str))]
    output_path: PathBuf,
    /// The maximum number of characters that should be written on a line.
    #[structopt(long)]
    column_span: Option<usize>,
}

pub fn run(options: &Options, _global: &GlobalOptions) -> CommandResult {
    log::debug!("running `minify`: {:?}", options);

    let resources = Resources::from_file_system();
    let process_options = darklua_core::Options::new(&options.input_path)
        .with_output(&options.output_path)
        .with_configuration(
            Configuration::empty().with_generator(
                options
                    .column_span
                    .map(|column_span| GeneratorParameters::Dense { column_span })
                    .unwrap_or_else(GeneratorParameters::default_dense),
            ),
        );

    let process_start_time = Instant::now();

    let result = darklua_core::process(&resources, process_options);

    let process_duration = durationfmt::to_string(process_start_time.elapsed());

    let success_count = result.success_count();

    match result.result() {
        Ok(()) => {
            println!(
                "successfully minified {} file{} (in {})",
                success_count,
                maybe_plural(success_count),
                process_duration
            );
            Ok(())
        }
        Err(errors) => {
            let error_count = errors.len();

            if success_count > 0 {
                eprintln!(
                    "successfully minified {} file{} (in {})",
                    success_count,
                    maybe_plural(success_count),
                    process_duration
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

            Err(CliError::new(1))
        }
    }
}
