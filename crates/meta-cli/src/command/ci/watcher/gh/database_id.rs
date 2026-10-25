use std::ffi::OsString;
use std::fmt;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct DatabaseId(u64);

impl DatabaseId {
    pub fn argument(self) -> OsString {
        OsString::from(self.to_string())
    }
}

impl fmt::Display for DatabaseId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
