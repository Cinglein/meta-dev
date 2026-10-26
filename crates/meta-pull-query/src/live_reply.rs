use serde::Deserialize;

use crate::pulls::Pulls;

#[derive(Clone, Debug, Deserialize)]
pub struct LiveReply {
    #[serde(default)]
    data: Option<Pulls>,
}

impl LiveReply {
    #[must_use]
    pub fn answered(self) -> Option<Pulls> {
        self.data
    }
}
