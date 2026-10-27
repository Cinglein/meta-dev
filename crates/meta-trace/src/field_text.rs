use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldText(String);

impl FieldText {
    pub fn shown(value: &dyn fmt::Debug) -> Self {
        Self(format!("{value:?}"))
    }

    pub fn titled(shown: &dyn fmt::Display) -> Self {
        Self(shown.to_string())
    }
}

impl fmt::Display for FieldText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
