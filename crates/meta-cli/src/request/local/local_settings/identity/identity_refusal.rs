use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum IdentityRefusal {
    #[error(
        "no identity: add [identity] name and email to meta.local.toml, or set git user.name and user.email (a GitHub noreply address) in this clone"
    )]
    Missing,
    #[error("git user.name {0}")]
    Name(&'static str),
    #[error("git user.email {0}")]
    Email(&'static str),
}
