use crate::cli::error::CliError;
use crate::cli::utils::{maybe_plural, write_file, Config, FileProcessing};
use crate::cli::GlobalOptions;

use darklua_core::{
    generator::{DenseLuaGenerator, LuaGenerator, ReadableLuaGenerator},
    nodes::Block,
    rules::Context,
    Parser,
};
use std::fs;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, Copy, Clone)]
pub enum LuaFormat {
    Dense,
    Readable,
}

impl LuaFormat {
    pub fn generate(&self, config: &Config, block: &Block) -> String {
        match self {
            Self::Dense => {
                let mut generator = DenseLuaGenerator::new(config.column_span);
                generator.write_block(block);
                generator.into_string()
            }
            Self::Readable => {
                let mut generator = ReadableLuaGenerator::new(config.column_span);
                generator.write_block(block);
                generator.into_string()
            }
        }
    }
}

impl FromStr for LuaFormat {
    type Err = String;

    fn from_str(format: &str) -> Result<Self, Self::Err> {
        match format {
            "dense" => Ok(Self::Dense),
            "readable" => Ok(Self::Readable),
            _ => Err(format!(
                "format '{}' does not exist! (possible options are: 'dense' or 'readable'",
                format
            )),
        }
    }
}

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
    /// Choose how Lua code is formatted ('dense' or 'readable').
    #[structopt(long, default_value = "dense")]
    pub format: LuaFormat,
}

fn process(
    file: &FileProcessing,
    options: &Options,
    global: &GlobalOptions,
) -> Result<(), CliError> {
    let config = Config::new(&options.config_path, global)?;

    let source = &file.source;
    let output = &file.output;

    if !source.exists() {
        return Err(CliError::InputFileNotFound(source.clone()));
    }

    let input = fs::read_to_string(source)
        .map_err(|io_error| CliError::InputFile(format!("{}", io_error)))?;

    let parser = Parser::default();

    let mut block = parser
        .parse(&input)
        .map_err(|parser_error| CliError::Parser(source.clone(), parser_error))?;

    for (index, rule) in config.process.iter().enumerate() {
        let mut context = Context::default();
        rule.process(&mut block, &mut context)
            .map_err(|rule_errors| CliError::RuleError {
                file: source.clone(),
                rule_name: rule.get_name().to_owned(),
                rule_number: index,
                errors: rule_errors,
            })?;
    }

    let lua_code = options.format.generate(&config, &block);

    write_file(output, &lua_code)
        .map_err(|io_error| CliError::OutputFile(output.clone(), format!("{}", io_error)))?;

    if global.verbose > 0 {
        println!("Successfully processed <{}>", source.to_string_lossy());
    };

    Ok(())
}

pub fn run(options: &Options, global: &GlobalOptions) {
    let files = FileProcessing::find(&options.input_path, &options.output_path, global);

    let results: Vec<Result<(), CliError>> = files
        .iter()
        .map(|file_processing| process(file_processing, options, global))
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
            "Successfully processed {} file{}",
            total_files,
            maybe_plural(total_files)
        );
    } else {
        let success_count = total_files - error_count;

        if success_count > 0 {
            eprintln!(
                "Successfully processed {} file{}.",
                success_count,
                maybe_plural(success_count)
            );
        }

        eprintln!(
            "But {} error{} happened:",
            error_count,
            maybe_plural(error_count)
        );

        errors.iter().for_each(|error| eprintln!("-> {}", error));
        process::exit(1);
    }
}
