use std::fmt;

use serde::{Deserialize, Serialize};

use crate::label::Label;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PointName(Label);

impl fmt::Display for PointName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
