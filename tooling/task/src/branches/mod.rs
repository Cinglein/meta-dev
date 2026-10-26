mod branch_name;

pub use branch_name::BranchName;

use enum_map::Enum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Branch {
    Default,
    Base,
}
