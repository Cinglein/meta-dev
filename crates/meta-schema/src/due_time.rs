use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
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
