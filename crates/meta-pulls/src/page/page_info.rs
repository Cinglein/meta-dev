mod continuation;

use serde::Deserialize;

use continuation::Continuation;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
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
