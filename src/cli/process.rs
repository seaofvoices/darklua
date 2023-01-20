use crate::cli::error::CliError;
use crate::cli::utils::maybe_plural;
use crate::cli::{CommandResult, GlobalOptions};

use clap::Args;
use darklua_core::{GeneratorParameters, Resources};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

#[derive(Debug, Args)]
pub struct Options {
    /// Path to the lua file to process.
    input_path: PathBuf,
    /// Where to output the result.
    output_path: PathBuf,
    /// Choose a specific configuration file.
    #[arg(long, short, alias = "config-path")]
    config: Option<PathBuf>,
    /// Choose how Lua code is formatted ('dense', 'readable' or 'retain-lines').
    /// This will override the format given by the configuration file.
    #[arg(long)]
    format: Option<LuaFormat>,
}

#[derive(Debug, Copy, Clone)]
enum LuaFormat {
    Dense,
    Readable,
    RetainLines,
}

impl FromStr for LuaFormat {
    type Err = String;

    fn from_str(format: &str) -> Result<Self, Self::Err> {
        match format {
            "dense" => Ok(Self::Dense),
            "readable" => Ok(Self::Readable),
            "retain-lines" => Ok(Self::RetainLines),
            _ => Err(format!(
                "format '{}' does not exist! (possible options are: 'dense', 'readable' or 'retain-lines'",
                format
            )),
        }
    }
}

pub fn run(options: &Options, _global: &GlobalOptions) -> CommandResult {
    log::debug!("running `process`: {:?}", options);

    let resources = Resources::from_file_system();
    let mut process_options =
        darklua_core::Options::new(&options.input_path).with_output(&options.output_path);

    if let Some(config) = options.config.as_ref() {
        process_options = process_options.with_configuration_at(config);
    }

    if let Some(format) = options.format.as_ref() {
        process_options = process_options.with_generator_override(match format {
            LuaFormat::Dense => GeneratorParameters::default_dense(),
            LuaFormat::Readable => GeneratorParameters::default_readable(),
            LuaFormat::RetainLines => GeneratorParameters::RetainLines,
        })
    }

    let process_start_time = Instant::now();

    let result = darklua_core::process(&resources, process_options);

    let process_duration = durationfmt::to_string(process_start_time.elapsed());

    let success_count = result.success_count();

    match result.result() {
        Ok(()) => {
            println!(
                "successfully processed {} file{} (in {})",
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
                    "successfully processed {} file{} (in {})",
                    success_count,
                    maybe_plural(success_count),
                    process_duration
                );
                eprintln!(
                    "but {} error{} happened:",
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
