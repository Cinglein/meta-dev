use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DuplicateLevel {
    Warn,
    Deny,
}
