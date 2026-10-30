use std::fmt;

use crate::identity::Identity;
use crate::object_id::ObjectId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflogLine {
    commit: ObjectId,
    parents: Vec<ObjectId>,
    committer: Identity,
    mover: Identity,
    moved: git2::Time,
    subject: String,
}

impl ReflogLine {
    const SEPARATOR: char = '\u{1f}';

    pub(crate) fn read(entry: &git2::ReflogEntry<'_>, commit: &git2::Commit<'_>) -> ReflogLine {
        let mover = entry.committer();
        ReflogLine {
            commit: ObjectId::from(commit.id()),
            parents: commit.parent_ids().map(ObjectId::from).collect(),
            committer: Identity::from(&commit.committer()),
            mover: Identity::from(&mover),
            moved: mover.when(),
            subject: String::from_utf8_lossy(entry.message_bytes().unwrap_or_default()).into_owned(),
        }
    }
}

impl fmt::Display for ReflogLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parents: Vec<String> = self.parents.iter().map(ToString::to_string).collect();
        let separator = Self::SEPARATOR;
        write!(
            formatter,
            "{}{separator}{}{separator}{}{separator}{} HEAD@{{{}}}{separator}{separator}{}",
            self.commit,
            parents.join(" "),
            self.committer,
            self.mover,
            self.moved.seconds(),
            self.subject
        )
    }
}
