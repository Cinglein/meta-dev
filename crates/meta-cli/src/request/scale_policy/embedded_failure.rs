#[derive(Clone, Debug, thiserror::Error)]
pub enum EmbeddedFailure {
    #[error("the embedded tooling/policy.toml cannot be parsed: {0}")]
    Unparsable(#[from] toml::de::Error),
}
