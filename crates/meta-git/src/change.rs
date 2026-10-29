use crate::git_failure::GitFailure;

pub trait Change {
    type Applied;

    fn applied(&self, repository: &git2::Repository) -> Result<Self::Applied, GitFailure>;
}
