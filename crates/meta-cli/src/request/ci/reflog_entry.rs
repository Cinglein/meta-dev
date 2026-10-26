mod identity;
mod move_subject;
mod stamp;

use std::str::FromStr;

use super::commit::Commit;
use super::start_moment::StartMoment;
use super::unix_seconds::UnixSeconds;
use identity::Identity;
use move_subject::{CommitAction, MoveSubject};
use stamp::Stamp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflogEntry {
    commit: Commit,
    parents: Vec<Commit>,
    committer: Identity,
    moved: Stamp,
    subject: MoveSubject,
}

impl ReflogEntry {
    pub const FORMAT: &'static str = "--format=%H%x1f%P%x1f%cn <%ce>%x1f%gn <%ge> %gd%x1f%gs";
    const SEPARATOR: char = '\u{1f}';

    pub fn created_after(
        &self,
        start: &StartMoment,
        before: Option<&Self>,
        born_after: impl FnOnce(&Commit) -> bool,
    ) -> Option<&Commit> {
        let action = self.subject.action()?;
        ((action != CommitAction::Merge
            || (self.parents.get(1).is_some() && self.parents.first() == before.map(|before| &before.commit)))
            && self.moved.by_after(&self.committer, start, || born_after(&self.commit)))
        .then_some(&self.commit)
    }
}

impl FromStr for ReflogEntry {
    type Err = ();

    fn from_str(line: &str) -> Result<Self, ()> {
        let fields: Vec<&str> = line.split(Self::SEPARATOR).collect();
        match fields.as_slice() {
            [sha, parents, committer, moved, subject] => Ok(Self {
                commit: sha.parse().map_err(drop)?,
                parents: parents.split_whitespace().filter_map(|parent| parent.parse().ok()).collect(),
                committer: committer.parse().map_err(drop)?,
                moved: moved.parse().map_err(drop)?,
                subject: subject.parse().map_err(drop)?,
            }),
            _ => Err(()),
        }
    }
}
