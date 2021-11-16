use std::time::Instant;

#[derive(Debug, Eq, PartialEq)]
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn now() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn duration_label(&self) -> String {
        let duration = self.start.elapsed();
        durationfmt::to_string(duration)
    }
}
