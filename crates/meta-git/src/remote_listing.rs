use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

use meta_run::Stderr;

use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::ref_name::RefName;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RemoteListing(BTreeMap<RefName, ObjectId>);

impl RemoteListing {
    const SEPARATOR: char = '\t';

    const HEADS: &'static str = "refs/heads/*";

    #[must_use]
    pub fn target(&self, name: &RefName) -> Option<ObjectId> {
        self.0.get(name).copied()
    }

    pub(crate) fn heads_at(path: &Path) -> Result<RemoteListing, Stderr> {
        let listed = git2::Repository::open(path).and_then(|repository| {
            repository.references_glob(Self::HEADS)?.try_fold(RemoteListing::default(), RemoteListing::entered)
        });
        listed.map_err(|error| Stderr::said(error.message().to_owned()))
    }

    fn entered(mut self, head: Result<git2::Reference<'_>, git2::Error>) -> Result<RemoteListing, git2::Error> {
        let head = head?;
        let Ok(name) = String::from_utf8_lossy(head.name_bytes()).parse();
        self.0.extend(head.resolve()?.target().map(|target| (name, ObjectId::from(target))));
        Ok(self)
    }
}

impl fmt::Display for RemoteListing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|(name, id)| writeln!(formatter, "{id}{}{name}", Self::SEPARATOR))
    }
}

impl FromStr for RemoteListing {
    type Err = GitFailure;

    fn from_str(listed: &str) -> Result<RemoteListing, GitFailure> {
        listed
            .lines()
            .map(|line| {
                let (id, name) =
                    line.split_once(Self::SEPARATOR).ok_or_else(|| GitFailure::NotObjectId(line.to_owned()))?;
                let Ok(name) = name.parse();
                Ok((name, id.parse()?))
            })
            .collect::<Result<_, GitFailure>>()
            .map(RemoteListing)
    }
}
