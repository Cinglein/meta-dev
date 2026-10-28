use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0:?} is not a 40-hex commit id")]
pub struct NotCommitId(String);

impl NotCommitId {
    pub(super) fn new(text: &str) -> Self {
        Self(text.to_owned())
    }
}
