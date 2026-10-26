use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum Unready {
    #[error("tree not clean")]
    Dirty,
    #[error("HEAD not pushed")]
    Unpushed,
    #[error("HEAD moved during the run")]
    Moved,
}
