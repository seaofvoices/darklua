use std::{
    path::{Component, Path, PathBuf},
    time::{Duration, Instant},
};

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Timer {
    start: Instant,
    accumulated_time: Duration,
}

impl Timer {
    pub fn now() -> Self {
        Self {
            start: Instant::now(),
            accumulated_time: Duration::new(0, 0),
        }
    }

    pub fn pause(&mut self) {
        let duration = self.start.elapsed();
        self.accumulated_time += duration;
    }

    pub fn start(&mut self) {
        self.start = Instant::now();
    }

    pub fn duration_label(&self) -> String {
        let duration = self.start.elapsed();
        durationfmt::to_string(duration + self.accumulated_time)
    }
}
