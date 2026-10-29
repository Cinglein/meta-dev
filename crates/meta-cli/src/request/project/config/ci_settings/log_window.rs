mod line_count;

use std::ops::RangeInclusive;

use serde::Deserialize;

use line_count::LineCount;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct LogWindow {
    log_lines: LineCount,
    log_context: LineCount,
}

impl LogWindow {
    pub fn limit(&self) -> LineCount {
        self.log_lines
    }

    pub fn index_span(&self, index: usize) -> RangeInclusive<usize> {
        index.saturating_sub(self.log_context.len())..=index.saturating_add(self.log_context.len())
    }
}
