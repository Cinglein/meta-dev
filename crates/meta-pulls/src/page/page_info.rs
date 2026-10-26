mod continuation;

use serde::Deserialize;

use continuation::Continuation;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    next: Continuation,
}

impl PageInfo {
    #[must_use]
    pub fn truncated(self) -> Option<()> {
        (self.next == Continuation::More).then_some(())
    }
}
