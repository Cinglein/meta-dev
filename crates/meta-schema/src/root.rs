use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Root {
    Process,
    State,
}
