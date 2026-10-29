use thiserror::Error;

#[derive(Debug, Error)]
#[error("missing; scales are not defaulted; restore it (git restore tooling/policy.toml) or create it")]
pub struct PolicyMissing;
