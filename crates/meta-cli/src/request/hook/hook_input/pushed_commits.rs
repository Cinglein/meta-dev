use std::fmt;
use std::path::{Path, PathBuf};

use meta_git::{Addition, FirstAddition, GitFailure, Reads as _, RemoteName, Repository};

use crate::request::output::Listed;

#[derive(Debug)]
pub struct PushedCommits {
    directory: PathBuf,
    remote: RemoteName,
}

impl PushedCommits {
    const SCRATCH: &'static str = concat!("/tmp/", "claude-");

    pub fn refusal_in(directory: &Path, remote: &RemoteName) -> Option<Listed> {
        let pushed = Self { directory: directory.to_owned(), remote: remote.clone() };
        match pushed.first_leak() {
            Ok(leak) => leak.map(|leak| Self::leaked(&leak)),
            Err(failure) => Some(Listed::new(format!("cannot read {pushed}: {failure}; not pushed"))),
        }
    }

    fn first_leak(&self) -> Result<Option<Addition>, GitFailure> {
        let leak = FirstAddition::new(self.remote.clone(), Self::SCRATCH.to_owned());
        Repository::open(&self.directory)?.read(&leak)
    }

    fn leaked(leak: &Addition) -> Listed {
        Listed::new(format!(
            "{leak}: agent scratch path in commit {}; remove it from that commit (amend or recommit; a later fix commit does not clear it); not pushed",
            leak.commit()
        ))
    }
}

impl fmt::Display for PushedCommits {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "the commits to push from {}", self.directory.display())
    }
}

#[cfg(test)]
mod tests {
    use super::PushedCommits;

    #[test]
    fn a_directory_git_cannot_read_holds_the_push_and_says_why() {
        let empty = tempfile::tempdir().unwrap();
        let held = PushedCommits::refusal_in(empty.path(), &"origin".parse().unwrap()).map(|why| why.to_string());
        assert!(
            held.is_some_and(|why| why.starts_with("cannot read the commits to push") && why.ends_with("not pushed"))
        );
    }
}
