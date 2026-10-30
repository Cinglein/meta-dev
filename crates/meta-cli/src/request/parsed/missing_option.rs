use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum MissingOption {
    #[error("--division <path> is required")]
    Division,
    #[error("--on <issue id|task id|review|ci|merge> is required")]
    On,
    #[error("--root <process|state> is required")]
    Root,
    #[error("--row <budget>:<measure><=<target> is required")]
    Row,
    #[error("--span <name> is required")]
    Span,
    #[error("--gate <budget:id|step:name|hook:name> is required")]
    Gate,
}
