use crate::git_failure::GitFailure;

pub trait Query {
    type View;

    fn answered(&self, repository: &git2::Repository) -> Result<Self::View, GitFailure>;
}
