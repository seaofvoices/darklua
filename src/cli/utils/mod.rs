mod config;

pub use config::*;

use crate::cli::GlobalOptions;

use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

pub const DEFAULT_COLUMN_SPAN: usize = 80;

#[derive(Debug, Clone)]
pub struct FileProcessing {
    pub source: PathBuf,
    pub output: PathBuf,
}

impl FileProcessing {
    pub fn find(input: &PathBuf, output: &PathBuf, global: &GlobalOptions) -> Vec<FileProcessing> {
        if input.is_file() {
            vec![FileProcessing {
                source: input.to_owned(),
                output: output.to_owned(),
            }]
        } else if input.is_dir() {
            let mut files = Vec::new();

            walk_dir(input, output, &mut files, global);

            files
        } else {
            Vec::new()
        }
    }
}

fn walk_dir(path: &PathBuf, output: &PathBuf, files: &mut Vec<FileProcessing>, global: &GlobalOptions) {
    let entries = fs::read_dir(path)
        .expect("error while reading directory");

    for entry in entries.into_iter() {
        let entry = entry.unwrap_or_else(|io_error| panic!(
            "error with entry (under {}): {}",
            path.to_string_lossy(),
            io_error
        ));

        let file_path = entry.path();

        if let Some(name) = file_path.file_name() {
            if file_path.is_dir() {
                let mut next_output = output.clone();
                next_output.push(name);
                walk_dir(&file_path, &next_output, files, global);

            } else if file_path.is_file() {
                match file_path.extension().and_then(OsStr::to_str) {
                    Some("lua") => {
                        let mut file_output = output.clone();
                        file_output.push(name);

                        files.push(FileProcessing {
                            source: file_path.clone(),
                            output: file_output,
                        });
                    }
                    _ => {}
                }
            } else {
                if global.verbose > 1 {
                    println!("Unexpected directory entry: {}", file_path.to_string_lossy());
                }
            }
        } else {
            if global.verbose > 1 {
                println!("No file name for path {}", file_path.to_string_lossy());
            }
        }
    }
}

/// Creates a file and the directories to it if they don't exist.
pub fn write_file(path: &Path, content: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    };

    let file = File::create(path)?;

    let mut file = BufWriter::new(file);
    file.write_all(content.as_bytes())
}

pub fn maybe_plural(count: usize) -> &'static str {
    if count > 1 { "s" } else { "" }
}
