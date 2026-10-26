use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    InProgress,
    Completed,
    Waiting,
    Requested,
    Pending,
    #[serde(other)]
    Unknown,
}

impl RunStatus {
    pub fn completed(self) -> Option<Self> {
        match self {
            Self::Completed => Some(self),
            Self::Queued | Self::InProgress | Self::Waiting | Self::Requested | Self::Pending | Self::Unknown => None,
        }
    }
}
