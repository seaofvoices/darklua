#[cfg(not(target_arch = "wasm32"))]
mod timer;
#[cfg(target_arch = "wasm32")]
mod wasm_timer;

#[cfg(not(target_arch = "wasm32"))]
pub use timer::Timer;
#[cfg(target_arch = "wasm32")]
pub use wasm_timer::Timer;

use std::path::{Component, Path, PathBuf};

pub fn maybe_plural(count: usize) -> &'static str {
    if count > 1 {
        "s"
    } else {
        ""
    }
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}
