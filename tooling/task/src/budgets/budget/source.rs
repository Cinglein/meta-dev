use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Source {
    #[default]
    Ci,
}
