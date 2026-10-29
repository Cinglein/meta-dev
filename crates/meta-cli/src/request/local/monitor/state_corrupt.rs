use thiserror::Error;

#[derive(Debug, Error)]
#[error("not valid monitor state ({0}); delete the state file to reset it")]
pub struct StateCorrupt(pub serde_json::Error);
