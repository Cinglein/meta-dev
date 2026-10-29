use serde::{Deserialize, Serialize};

use crate::budget::Millis;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Liveness {
    silent_after: Millis,
    reply_within: Millis,
}
