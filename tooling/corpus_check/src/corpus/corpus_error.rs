use std::io;

use thiserror::Error;

use task::Failure;
use task::Site;

#[derive(Debug, Error)]
pub enum CorpusError {
    #[error("{site}: {error}")]
    Io { site: Site, error: io::Error },
    #[error(transparent)]
    Source(#[from] Failure),
    #[error("marker `//~{text}` is not `//~ lint: message`, `//~ file|crate|workspace lint: message`")]
    Marker { text: String },
}
