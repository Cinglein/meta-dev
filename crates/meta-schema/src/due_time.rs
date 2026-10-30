use std::fmt;
use std::str::FromStr;

use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use crate::unrecognised::Unrecognised;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DueTime(DateTime<Utc>);

impl From<DateTime<Utc>> for DueTime {
    fn from(time: DateTime<Utc>) -> Self {
        Self(time)
    }
}

impl DueTime {
    #[must_use]
    pub fn elapsed_since(self, earlier: Self) -> Option<Duration> {
        (self.0 - earlier.0).to_std().ok()
    }

    #[must_use]
    pub fn after(self, wait: Duration) -> Option<Self> {
        let wait = TimeDelta::from_std(wait).ok()?;
        self.0.checked_add_signed(wait).map(Self)
    }
}

impl FromStr for DueTime {
    type Err = Unrecognised;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        DateTime::parse_from_rfc3339(text)
            .map(|time| Self(time.with_timezone(&Utc)))
            .map_err(|_| Unrecognised::new("an RFC 3339 time", text))
    }
}

impl fmt::Display for DueTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.format("%Y-%m-%d %H:%MZ"))
    }
}
