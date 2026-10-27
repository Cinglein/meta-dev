mod warning_failure;

use serde::{Deserialize, Serialize};

use crate::policy::percent::Percent;
pub use warning_failure::WarningFailure;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Percent", into = "Percent")]
pub struct WarningLine(Percent);

impl WarningLine {
    const DEFAULT: Percent = Percent::new(60);
    const BLOCKING: Percent = Percent::new(100);

    #[must_use]
    pub fn percent(self) -> Percent {
        self.0
    }
}

impl Default for WarningLine {
    fn default() -> WarningLine {
        WarningLine(Self::DEFAULT)
    }
}

impl TryFrom<Percent> for WarningLine {
    type Error = WarningFailure;

    fn try_from(percent: Percent) -> Result<WarningLine, WarningFailure> {
        if percent < Self::BLOCKING {
            Ok(WarningLine(percent))
        } else {
            Err(WarningFailure::AtTheBlockingLine(percent))
        }
    }
}

impl From<WarningLine> for Percent {
    fn from(line: WarningLine) -> Percent {
        line.0
    }
}
