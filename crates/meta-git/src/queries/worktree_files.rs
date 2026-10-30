use std::path::PathBuf;

use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::query::Query;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorktreeFiles {
    ChangedSince(ObjectId),
    Untracked,
    Unclean,
    Touched,
    Tracked,
}

impl WorktreeFiles {
    fn statused(repository: &git2::Repository, flags: git2::Status) -> Result<Vec<PathBuf>, git2::Error> {
        let mut options = git2::StatusOptions::new();
        let ignored = flags.intersects(git2::Status::IGNORED);
        options
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(ignored)
            .recurse_ignored_dirs(ignored);
        let statuses = repository.statuses(Some(&mut options))?;
        let wanted = statuses.iter().filter(|entry| entry.status().intersects(flags));
        Ok(wanted.map(|entry| PathBuf::from(String::from_utf8_lossy(entry.path_bytes()).into_owned())).collect())
    }

    fn changed(repository: &git2::Repository, commit: ObjectId) -> Result<Vec<PathBuf>, git2::Error> {
        let tree = repository.find_commit(commit.oid())?.tree()?;
        let diff = repository.diff_tree_to_workdir_with_index(Some(&tree), None)?;
        let paths = diff.deltas().filter_map(|delta| delta.new_file().path().or_else(|| delta.old_file().path()));
        Ok(paths.map(PathBuf::from).collect())
    }

    fn indexed(repository: &git2::Repository) -> Result<Vec<PathBuf>, git2::Error> {
        let index = repository.index()?;
        Ok(index.iter().map(|entry| PathBuf::from(String::from_utf8_lossy(&entry.path).into_owned())).collect())
    }
}

impl Query for WorktreeFiles {
    type View = Vec<PathBuf>;

    fn answered(&self, repository: &git2::Repository) -> Result<Vec<PathBuf>, GitFailure> {
        Ok(match self {
            WorktreeFiles::ChangedSince(commit) => Self::changed(repository, *commit)?,
            WorktreeFiles::Untracked => Self::statused(repository, git2::Status::WT_NEW)?,
            WorktreeFiles::Unclean => {
                Self::statused(repository, git2::Status::all().difference(git2::Status::IGNORED))?
            }
            WorktreeFiles::Touched => Self::statused(repository, git2::Status::all())?,
            WorktreeFiles::Tracked => Self::indexed(repository)?,
        })
    }
}
