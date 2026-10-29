use crate::change::Change;
use crate::git_failure::GitFailure;

pub trait Writes {
    fn write<Made: Change>(&self, change: &Made) -> Result<Made::Applied, GitFailure>;
}
