use std::path::{Path, PathBuf};

pub(crate) fn find_require_paths<'a, 'b, 'c>(
    path: &'a Path,
    module_folder_name: &'b str,
) -> impl Iterator<Item = PathBuf> + 'c
where
    'a: 'c,
    'b: 'c,
{
    PathIterator::new(path, module_folder_name)
}

struct PathIterator<'a, 'b> {
    path: &'a Path,
    has_extension: bool,
    module_folder_name: &'b str,
    index: u8,
}

impl<'a, 'b> PathIterator<'a, 'b> {
    fn new(path: &'a Path, module_folder_name: &'b str) -> Self {
        Self {
            path,
            has_extension: path.extension().is_some(),
            module_folder_name,
            index: 0,
        }
    }

    fn return_next(&mut self, path: PathBuf) -> Option<PathBuf> {
        self.index += 1;
        Some(path)
    }
}

impl Iterator for PathIterator<'_, '_> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_extension {
            match self.index {
                0 => self.return_next(self.path.to_path_buf()),
                _ => None,
            }
        } else {
            match self.index {
                0 => self.return_next(self.path.to_path_buf()),
                1 => self.return_next(self.path.with_extension("luau")),
                2 => self.return_next(self.path.with_extension("lua")),
                3 => self.return_next(self.path.join(self.module_folder_name)),
                4 | 5 => {
                    let mut next_path = self.path.join(self.module_folder_name);
                    if next_path.extension().is_some() {
                        None
                    } else {
                        next_path.set_extension(if self.index == 4 { "luau" } else { "lua" });
                        self.return_next(next_path)
                    }
                }
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const ANY_FOLDER_NAME: &str = "test";
    const ANY_FOLDER_NAME_WITH_EXTENSION: &str = "test.luau";

    #[test]
    fn returns_exact_path_when_path_has_an_extension() {
        let source = Path::new("hello.lua");
        let iterator = PathIterator::new(source, ANY_FOLDER_NAME);

        pretty_assertions::assert_eq!(vec![source.to_path_buf()], iterator.collect::<Vec<_>>())
    }

    #[test]
    fn returns_paths_when_path_has_no_extension() {
        let source = Path::new("hello");
        let iterator = PathIterator::new(source, ANY_FOLDER_NAME);

        pretty_assertions::assert_eq!(
            vec![
                source.to_path_buf(),
                source.with_extension("luau"),
                source.with_extension("lua"),
                source.join(ANY_FOLDER_NAME),
                source.join(ANY_FOLDER_NAME).with_extension("luau"),
                source.join(ANY_FOLDER_NAME).with_extension("lua"),
            ],
            iterator.collect::<Vec<_>>()
        )
    }

    #[test]
    fn returns_paths_when_path_has_no_extension_and_module_folder_name_has_an_extension() {
        let source = Path::new("hello");
        let iterator = PathIterator::new(source, ANY_FOLDER_NAME_WITH_EXTENSION);

        pretty_assertions::assert_eq!(
            vec![
                source.to_path_buf(),
                source.with_extension("luau"),
                source.with_extension("lua"),
                source.join(ANY_FOLDER_NAME_WITH_EXTENSION),
            ],
            iterator.collect::<Vec<_>>()
        )
    }
}
