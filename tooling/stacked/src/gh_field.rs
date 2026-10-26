use std::fmt;

use crate::gh_key::GhKey;

pub struct GhField {
    key: GhKey,
    value: String,
}

impl GhField {
    pub fn new(key: GhKey, value: impl fmt::Display) -> GhField {
        GhField { key, value: value.to_string() }
    }
}

impl fmt::Display for GhField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}={}", self.key, self.value)
    }
}
