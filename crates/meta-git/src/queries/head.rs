use crate::checked_out::CheckedOut;
use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::query::Query;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Head;

impl Head {
    const NAME: &'static str = "HEAD";
    const BRANCHES: &'static str = "refs/heads/";
}

impl Query for Head {
    type View = CheckedOut;

    fn answered(&self, repository: &git2::Repository) -> Result<CheckedOut, GitFailure> {
        let head = repository.find_reference(Self::NAME)?;
        let branch = head.symbolic_target()?.filter(|target| target.starts_with(Self::BRANCHES));
        let Ok(branch) = branch.map(str::parse).transpose();
        let missing = |error: git2::Error| {
            (error.code() == git2::ErrorCode::UnbornBranch || error.code() == git2::ErrorCode::NotFound)
                .then_some(None)
                .ok_or(error)
        };
        let commit = head.resolve().map(|resolved| resolved.target().map(ObjectId::from)).or_else(missing)?;
        Ok(CheckedOut::new(commit, branch))
    }
}
