use std::path::PathBuf;

use super::held::Held;

#[derive(Debug, PartialEq, Eq)]
pub enum Scan {
    Opened(Vec<PathBuf>),
    Skipped,
    Uninspected,
}

impl Scan {
    pub(crate) fn added(self, held: Held) -> Held {
        match self {
            Self::Opened(paths) => held.holding(paths),
            Self::Skipped => held,
            Self::Uninspected => held.refused(),
        }
    }
}
