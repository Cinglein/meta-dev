use std::io;

use thiserror::Error;

use crate::ratchet::Ratchet;
use crate::site::Site;

#[derive(Debug, Error)]
pub enum Failure {
    #[error("{site}: {error}")]
    Io { site: Site, error: io::Error },
    #[error("{site}: {error}")]
    Parse { site: Site, error: syn::Error },
    #[error("{site}: {error}")]
    Baseline { site: Site, error: Box<toml::de::Error> },
    #[error("baseline: {0}")]
    Serialize(toml::ser::Error),
    #[error("{0}")]
    Ratchet(Box<Ratchet>),
}
