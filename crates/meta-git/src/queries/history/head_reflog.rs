use super::reflog_line::ReflogLine;
use crate::git_failure::GitFailure;
use crate::query::Query;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeadReflog;

impl HeadReflog {
    const HEAD: &'static str = "HEAD";
}

impl Query for HeadReflog {
    type View = Vec<ReflogLine>;

    fn answered(&self, repository: &git2::Repository) -> Result<Vec<ReflogLine>, GitFailure> {
        let reflog = repository.reflog(Self::HEAD)?;
        let lines = reflog.iter().filter_map(|entry| {
            let commit = repository.find_commit(entry.id_new()).ok()?;
            Some(ReflogLine::read(&entry, &commit))
        });
        Ok(lines.collect())
    }
}
