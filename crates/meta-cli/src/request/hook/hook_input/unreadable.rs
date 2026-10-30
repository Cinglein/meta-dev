use crate::request::output::Listed;

#[derive(Debug, thiserror::Error)]
pub enum Unreadable {
    #[error("cannot read the hook input, so no command is allowed: {0}")]
    Io(#[from] std::io::Error),
    #[error("cannot parse the hook input, so no command is allowed: {0}")]
    Malformed(#[from] serde_json::Error),
}

impl From<Unreadable> for Listed {
    fn from(unreadable: Unreadable) -> Listed {
        Listed::new(unreadable)
    }
}
