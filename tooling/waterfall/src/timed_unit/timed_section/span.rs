use serde::Deserialize;

use crate::seconds::Seconds;

#[derive(Clone, Copy, Debug, Deserialize)]
pub(super) struct Span {
    start: Seconds,
    end: Seconds,
}

impl Span {
    pub(super) fn length(self) -> Seconds {
        self.end - self.start
    }
}
