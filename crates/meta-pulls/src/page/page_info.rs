mod continuation;
mod cursor;

use serde::Deserialize;

use continuation::Continuation;
pub use cursor::Cursor;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    next: Continuation,
    #[serde(default, rename = "endCursor")]
    cursor: Option<Cursor>,
}

impl PageInfo {
    #[must_use]
    pub fn truncated(&self) -> Option<()> {
        (self.next == Continuation::More).then_some(())
    }

    #[must_use]
    pub fn following(&self) -> Option<Cursor> {
        self.truncated().and(self.cursor.clone())
    }
}
