use serde::{Deserialize, Serialize};

use crate::budget::Millis;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Window {
    PerChange,
    Rolling(Millis),
    Standing,
}
