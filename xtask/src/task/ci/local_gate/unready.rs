use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum Unready {
    #[error("tree not clean")]
    Dirty,
    #[error("HEAD not pushed")]
    Unpushed,
    #[error("gate checkout changed during the run")]
    Tampered,
    #[error("no meta-dev rev")]
    Unversioned,
    #[error("the gate did not record every step as passed")]
    Incomplete,
}
