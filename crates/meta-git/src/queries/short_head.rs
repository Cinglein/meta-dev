use crate::git_failure::GitFailure;
use crate::query::Query;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShortHead;

impl Query for ShortHead {
    type View = String;

    fn answered(&self, repository: &git2::Repository) -> Result<String, GitFailure> {
        let short = repository.head()?.peel_to_commit()?.as_object().short_id()?;
        Ok(String::from_utf8_lossy(&short).into_owned())
    }
}
