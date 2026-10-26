use serde::Deserialize;

use crate::live_data::LiveData;

#[derive(Clone, Debug, Deserialize)]
pub struct LiveReply {
    #[serde(default)]
    data: Option<LiveData>,
}

impl LiveReply {
    #[must_use]
    pub fn answered(self) -> Option<LiveData> {
        self.data
    }
}
