use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Failure {
    #[error("ci: {step}: {error}")]
    Spawn { step: String, error: io::Error },
    #[error("ci: {0} failed")]
    Step(String),
}
