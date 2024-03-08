use crate::cli::{CommandResult, GlobalOptions};

use clap::Args;
use darklua_core::{Configuration, DarkluaError, Resources};
use std::path::PathBuf;

use super::error::CliError;

#[derive(Debug, Args)]
pub struct Options {
    /// Path where to write the JSON schema for darklua
    output: Option<PathBuf>,
}

pub fn run(options: &Options, _: &GlobalOptions) -> CommandResult {
    let resources = Resources::from_file_system();

    let schema = Configuration::schema();

    if let Some(output) = &options.output {
        resources.write(output, &schema).map_err(|err| {
            eprintln!("an error happened: {}", DarkluaError::from(err));
            CliError::new(1)
        })?;
    } else {
        println!("{}", schema);
    }

    Ok(())
}
