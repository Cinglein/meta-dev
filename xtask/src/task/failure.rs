use thiserror::Error;

#[derive(Debug, Error)]
pub enum Failure {
    #[error("{0}")]
    Task(#[from] task::Failure),
    #[error("{0}")]
    Kept(#[from] meta_lints::Failure),
}
