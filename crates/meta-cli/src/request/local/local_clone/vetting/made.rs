use super::kept::Kept;
use super::thaw::Thaw;

// The receipt of one Change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Made {
    Git(Result<String, Kept>),
    Frozen(Result<Thaw, Kept>),
    Thawed,
}
