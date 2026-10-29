use std::path::PathBuf;

use super::held::Held;

#[derive(Debug, PartialEq, Eq)]
pub enum Scan {
    Opened(Vec<PathBuf>),
    Skipped,
    #[cfg_attr(not(any(target_os = "linux", test)), expect(dead_code, reason = "only procfs counts these"))]
    Uninspected,
}

impl Scan {
    pub fn added(self, held: Held) -> Held {
        match self {
            Self::Opened(paths) => held.holding(paths),
            Self::Skipped => held,
            Self::Uninspected => held.refused(),
        }
    }
}
