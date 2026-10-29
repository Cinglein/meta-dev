mod branch_name;
mod trusted;

pub use branch_name::BranchName;
pub use trusted::Trusted;

use enum_map::Enum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Branch {
    Default,
    Base,
}
