use std::{
    ffi::OsStr,
    iter::FromIterator,
    path::{Component, Path, PathBuf},
};

pub(crate) fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    normalize(path, false)
}

pub(crate) fn normalize_path_with_current_dir(path: impl AsRef<Path>) -> PathBuf {
    normalize(path, true)
}

fn normalize(path: impl AsRef<Path>, keep_current_dir: bool) -> PathBuf {
    let mut components = path.as_ref().components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        vec![c.as_os_str()]
    } else {
        Vec::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {
                if keep_current_dir && ret.is_empty() {
                    ret.push(OsStr::new("."));
                }
            }
            Component::ParentDir => {
                if ret.last().filter(|c| **c != OsStr::new("..")).is_some() {
                    ret.pop();
                } else {
                    ret.push(OsStr::new(".."));
                }
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }

    PathBuf::from_iter(ret)
}

#[cfg(test)]
mod test {
    use super::*;

    fn verify_normalize_path(input: impl AsRef<Path>, output: impl AsRef<Path>) {
        assert_eq!(normalize_path(input.as_ref()), output.as_ref());
    }

    #[test]
    fn current_directory_with_name() {
        verify_normalize_path("./directory", "directory")
    }

    #[test]
    fn from_directory() {
        verify_normalize_path("/directory", "/directory")
    }

    #[test]
    fn parent_directory_path() {
        verify_normalize_path("..", "..")
    }

    #[test]
    fn src_parent_with_directory_name() {
        verify_normalize_path("src/../directory", "directory")
    }

    #[test]
    fn src_current_dir_with_directory_name() {
        verify_normalize_path("src/./directory", "src/directory")
    }

    #[test]
    fn double_parent_directory_path() {
        verify_normalize_path("../..", "../..")
    }

    mod with_current_dir {
        use super::*;

        fn verify_normalize_path_with_current_dir(
            input: impl AsRef<Path>,
            output: impl AsRef<Path>,
        ) {
            assert_eq!(
                normalize_path_with_current_dir(input.as_ref()),
                output.as_ref()
            );
        }

        #[test]
        fn current_directory_with_name() {
            verify_normalize_path_with_current_dir("./directory", "./directory")
        }

        #[test]
        fn src_parent_with_directory_name_from_current_directory() {
            verify_normalize_path_with_current_dir("./src/../directory", "./directory")
        }
    }
}
