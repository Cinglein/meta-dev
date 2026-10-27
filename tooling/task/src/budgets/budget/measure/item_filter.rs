use serde::{Deserialize, Serialize};
use strum::Display;

use super::item_kind::ItemKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ItemFilter {
    #[strum(to_string = "{0}s")]
    Kind(ItemKind),
    InDiscussion,
    UnheldTasks,
}
