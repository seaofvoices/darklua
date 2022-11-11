use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufWriter, ErrorKind as IOErrorKind, Write},
    iter,
    path::{Path, PathBuf},
};

use super::utils::normalize_path;

#[derive(Debug, Clone)]
enum Source {
    FileSystem,
    Memory(RefCell<HashMap<PathBuf, String>>),
}

impl Source {
    pub fn exists(&self, location: &Path) -> ResourceResult<bool> {
        match self {
            Self::FileSystem => Ok(location.exists()),
            Self::Memory(data) => Ok(data.borrow().contains_key(&normalize_path(location))),
        }
    }

    pub fn is_directory(&self, location: &Path) -> ResourceResult<bool> {
        let is_directory = match self {
            Source::FileSystem => self.exists(location)? && location.is_dir(),
            Source::Memory(data) => {
                let data = data.borrow();
                let location = normalize_path(location);

                data.iter()
                    .any(|(path, _content)| path.starts_with(&location))
            }
        };
        Ok(is_directory)
    }

    pub fn is_file(&self, location: &Path) -> ResourceResult<bool> {
        let is_file = match self {
            Source::FileSystem => self.exists(location)? && location.is_file(),
            Source::Memory(data) => {
                let data = data.borrow();
                let location = normalize_path(location);

                data.contains_key(&location)
            }
        };
        Ok(is_file)
    }

    pub fn get(&self, location: &Path) -> ResourceResult<String> {
        match self {
            Self::FileSystem => fs::read_to_string(location).map_err(|err| match err.kind() {
                IOErrorKind::NotFound => ResourceError::not_found(location),
                _ => ResourceError::io_error(location, err),
            }),
            Self::Memory(data) => {
                let data = data.borrow();
                let location = normalize_path(location);

                data.get(&location)
                    .map(String::from)
                    .ok_or_else(|| ResourceError::not_found(location))
            }
        }
    }

    pub fn write(&self, location: &Path, content: &str) -> ResourceResult<()> {
        match self {
            Self::FileSystem => {
                if let Some(parent) = location.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|err| ResourceError::io_error(parent, err))?;
                };

                let file =
                    File::create(location).map_err(|err| ResourceError::io_error(location, err))?;

                let mut file = BufWriter::new(file);
                file.write_all(content.as_bytes())
                    .map_err(|err| ResourceError::io_error(location, err))
            }
            Self::Memory(data) => {
                let mut data = data.borrow_mut();
                data.insert(normalize_path(location), content.to_string());
                Ok(())
            }
        }
    }

    pub fn walk(&self, location: &Path) -> impl Iterator<Item = PathBuf> {
        match self {
            Self::FileSystem => Box::new(walk_file_system(location.to_path_buf()))
                as Box<dyn Iterator<Item = PathBuf>>,
            Self::Memory(data) => {
                let data = data.borrow();
                let location = normalize_path(location);
                let mut paths: Vec<_> = data.keys().map(|path| normalize_path(path)).collect();
                paths.retain(|path| path.starts_with(&location));

                Box::new(paths.into_iter())
            }
        }
    }
}

fn walk_file_system(location: PathBuf) -> impl Iterator<Item = PathBuf> {
    let mut unknown_paths = vec![location];
    let mut file_paths = Vec::new();
    let mut dir_entries = Vec::new();

    iter::from_fn(move || loop {
        if let Some(location) = unknown_paths.pop() {
            match location.metadata() {
                Ok(metadata) => {
                    if metadata.is_file() {
                        file_paths.push(location.to_path_buf());
                    } else if metadata.is_dir() {
                        dir_entries.push(location.to_path_buf());
                    } else if metadata.is_symlink() {
                        log::warn!("unexpected symlink `{}` not followed", location.display());
                    } else {
                        log::warn!(
                            concat!(
                                "path `{}` points to an unexpected location that is not a ",
                                "file, not a directory and not a symlink"
                            ),
                            location.display()
                        );
                    };
                }
                Err(err) => {
                    log::warn!(
                        "unable to read metadata from file `{}`: {}",
                        location.display(),
                        err
                    );
                }
            }
        } else if let Some(dir_location) = dir_entries.pop() {
            match dir_location.read_dir() {
                Ok(read_dir) => {
                    for entry in read_dir {
                        match entry {
                            Ok(entry) => {
                                unknown_paths.push(entry.path());
                            }
                            Err(err) => {
                                log::warn!(
                                    "unable to read directory entry `{}`: {}",
                                    dir_location.display(),
                                    err
                                );
                            }
                        }
                    }
                }
                Err(err) => {
                    log::warn!(
                        "unable to read directory `{}`: {}",
                        dir_location.display(),
                        err
                    );
                }
            }
        } else if let Some(path) = file_paths.pop() {
            break Some(path);
        } else {
            break None;
        }
    })
}

#[derive(Debug, Clone)]
pub struct Resources {
    source: Source,
}

impl Resources {
    pub fn from_file_system() -> Self {
        Self {
            source: Source::FileSystem,
        }
    }

    pub fn from_memory() -> Self {
        Self {
            source: Source::Memory(Default::default()),
        }
    }

    pub fn collect_work(&self, location: impl AsRef<Path>) -> impl Iterator<Item = PathBuf> {
        self.source.walk(location.as_ref()).filter(|path| {
            matches!(
                path.extension().and_then(OsStr::to_str),
                Some("lua") | Some("luau")
            )
        })
    }

    pub fn exists(&self, location: impl AsRef<Path>) -> ResourceResult<bool> {
        self.source.exists(location.as_ref())
    }

    pub fn is_directory(&self, location: impl AsRef<Path>) -> ResourceResult<bool> {
        self.source.is_directory(location.as_ref())
    }

    pub fn is_file(&self, location: impl AsRef<Path>) -> ResourceResult<bool> {
        self.source.is_file(location.as_ref())
    }

    pub fn get(&self, location: impl AsRef<Path>) -> ResourceResult<String> {
        self.source.get(location.as_ref())
    }

    pub fn write(&self, location: impl AsRef<Path>, content: &str) -> ResourceResult<()> {
        self.source.write(location.as_ref(), content)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    NotFound(PathBuf),
    IO { path: PathBuf, error: String },
}

impl ResourceError {
    pub(crate) fn not_found(path: impl Into<PathBuf>) -> Self {
        Self::NotFound(path.into())
    }

    pub(crate) fn io_error(path: impl Into<PathBuf>, error: io::Error) -> Self {
        Self::IO {
            path: path.into(),
            error: error.to_string(),
        }
    }
}

type ResourceResult<T> = Result<T, ResourceError>;

#[cfg(test)]
mod test {
    use super::*;

    fn any_path() -> &'static Path {
        Path::new("test.lua")
    }

    const ANY_CONTENT: &str = "return true";

    mod memory {
        use std::iter::FromIterator;

        use super::*;

        fn new() -> Resources {
            Resources::from_memory()
        }

        #[test]
        fn not_created_file_does_not_exist() {
            assert_eq!(new().exists(any_path()), Ok(false));
        }

        #[test]
        fn created_file_exists() {
            let resources = new();
            resources.write(any_path(), ANY_CONTENT).unwrap();

            assert_eq!(resources.exists(any_path()), Ok(true));
        }

        #[test]
        fn read_content_of_created_file() {
            let resources = new();
            resources.write(any_path(), ANY_CONTENT).unwrap();

            assert_eq!(resources.get(any_path()), Ok(ANY_CONTENT.to_string()));
        }

        #[test]
        fn collect_work_contains_created_files() {
            let resources = new();
            resources.write("src/test.lua", ANY_CONTENT).unwrap();

            assert_eq!(
                Vec::from_iter(resources.collect_work("src")),
                vec![PathBuf::from("src/test.lua")]
            );
        }
    }
}
