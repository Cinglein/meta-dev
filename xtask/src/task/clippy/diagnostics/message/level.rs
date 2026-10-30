use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub(super) enum Level {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "error: internal compiler error")]
    Internal,
    #[serde(other)]
    Other,
}
