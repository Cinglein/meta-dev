mod identity;
mod move_subject;
mod stamp;

use std::str::FromStr;

use super::commit::Commit;
use super::start_moment::StartMoment;
use super::unix_seconds::UnixSeconds;
use crate::request::checkout::Checkout;
use identity::Identity;
use move_subject::{CommitAction, MoveSubject};
use stamp::Stamp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflogEntry {
    commit: Commit,
    parents: Vec<Commit>,
    committer: Identity,
    moved: Stamp,
    checkout: Option<Checkout>,
    subject: MoveSubject,
}

impl ReflogEntry {
    pub const WALK: [&'static str; 9] = [
        "log",
        "--walk-reflogs",
        "--date=unix",
        "--decorate=full",
        "--decorate-refs=HEAD",
        "--decorate-refs=refs/heads/",
        "--format=%H%x1f%P%x1f%cn <%ce>%x1f%gn <%ge> %gd%x1f%D%x1f%gs",
        "HEAD",
        "--",
    ];
    const SEPARATOR: char = '\u{1f}';

    pub fn checkout(&self) -> Option<&Checkout> {
        self.checkout.as_ref()
    }

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
            [sha, parents, committer, moved, decorated, subject] => {
                let commit: Commit = sha.parse().map_err(drop)?;
                Ok(Self {
                    checkout: Checkout::decorated(&commit, decorated),
                    commit,
                    parents: parents.split_whitespace().filter_map(|parent| parent.parse().ok()).collect(),
                    committer: committer.parse().map_err(drop)?,
                    moved: moved.parse().map_err(drop)?,
                    subject: subject.parse().map_err(drop)?,
                })
            }
            _ => Err(()),
        }
    }
}
