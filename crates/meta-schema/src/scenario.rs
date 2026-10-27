use std::fmt;

use serde::{Deserialize, Serialize};

use crate::text::Text;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Scenario(Text);

impl From<Text> for Scenario {
    fn from(text: Text) -> Scenario {
        Scenario(text)
    }
}

impl fmt::Display for Scenario {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
