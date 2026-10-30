#[derive(Clone, Debug, thiserror::Error)]
pub enum EmbeddedFailure {
    #[error("the embedded tooling/policy.toml cannot be parsed: {0}")]
    Unparsable(#[from] toml::de::Error),
    #[error("the embedded tooling/policy.toml lacks a required section ([tools] or [dispatch])")]
    Missing,
}
