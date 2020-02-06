use crate::cli::GlobalOptions;
use crate::cli::utils::{
    maybe_plural,
    write_file,
    FileProcessing,
};

use darklua_core::{LuaGenerator, ToLua, Parser, ParsingError};
use std::fmt::{self, Display};
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
    /// Amount of characters before the code is wrapped into a new line.
    #[structopt(long, short, default_value = "80")]
    pub column_span: usize,
}

pub enum Error {
    Parser(PathBuf, ParsingError),
    InputFile(String),
    InputFileNotFound(PathBuf),
    OutputFile(PathBuf, String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parser(path, error) => {
                write!(f, "could not parse input at <{}>: {:?}", path.to_string_lossy(), error)
            }
            Self::InputFile(error) => {
                write!(f, "error while reading input file: {}", error)
            }
            Self::InputFileNotFound(path) => {
                write!(f, "input file not found: {}", path.to_string_lossy())
            }
            Self::OutputFile(path, error) => {
                write!(f, "error with output file <{}>: {}", path.to_string_lossy(), error)
            }
        }
    }
}

type MinifyResult = Result<(), Error>;

fn process(file: &FileProcessing, options: &Options, global: &GlobalOptions) -> MinifyResult {
    let source = &file.source;
    let output = &file.output;

    if !source.exists() {
        return Err(Error::InputFileNotFound(source.clone()))
    }

    let input = fs::read_to_string(source)
        .map_err(|io_error| Error::InputFile(format!("{}", io_error)))?;

    let parser = Parser::default();

    let block = parser.parse(&input)
        .map_err(|parser_error| Error::Parser(source.clone(), parser_error))?;

    let mut generator = LuaGenerator::new(options.column_span);
    block.to_lua(&mut generator);
    let minified = generator.into_string();

    write_file(&output, &minified)
        .map_err(|io_error| Error::OutputFile(output.clone(), format!("{}", io_error)))?;

    if global.verbose > 0 {
        println!("Successfully processed <{}>", source.to_string_lossy());
    };

    Ok(())
}

pub fn run(options: &Options, global: &GlobalOptions) {
    let file = FileProcessing::find(&options.input_path, &options.output_path, global);

    let results: Vec<Result<(), Error>> = file.iter()
        .map(|file_processing| process(file_processing, &options, global))
        .collect();

    let total_files = results.len();

    let errors: Vec<Error> = results.into_iter()
        .filter_map(|result| match result {
            Ok(()) => {
                None
            }
            Err(error) => Some(error),
        })
        .collect();

    let error_count = errors.len();

    if error_count == 0 {
        println!("Successfully minified {} file{}", total_files, maybe_plural(total_files));

    } else {
        let success_count = total_files - error_count;

        if success_count > 0 {
            eprintln!("Successfully minified {} file{}.", success_count, maybe_plural(success_count));
        }

        eprintln!("But {} error{} happened:", error_count, maybe_plural(error_count));

        errors.iter().for_each(|error| eprintln!("-> {}", error));
        process::exit(1);
    }
}
