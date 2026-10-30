use std::fmt;
use std::path::Path;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use super::unix_seconds::UnixSeconds;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartMoment(Duration);

impl StartMoment {
    pub fn now() -> Self {
        Self(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default())
    }

    pub fn against_second(&self, moment: UnixSeconds) -> std::cmp::Ordering {
        moment.cmp(&UnixSeconds::from(self.0))
    }

    pub fn precedes(&self, time: SystemTime) -> bool {
        time.duration_since(SystemTime::UNIX_EPOCH).is_ok_and(|since| self.0.lt(&since))
    }

    pub fn born_after(&self, object: &Path, modified: impl FnOnce(&Path) -> Option<SystemTime>) -> bool {
        modified(object).is_some_and(|modified| self.precedes(modified))
    }
}

impl FromStr for StartMoment {
    type Err = std::num::ParseIntError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.trim().parse().map(Duration::from_nanos).map(Self)
    }
}

impl fmt::Display for StartMoment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.as_nanos())
    }
}
