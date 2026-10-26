mod move_subject;
mod stamp;

use std::str::FromStr;

use super::commit::Commit;
use super::unix_seconds::UnixSeconds;
use move_subject::MoveSubject;
use stamp::Stamp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflogEntry {
    commit: Commit,
    committed: Stamp,
    moved: Stamp,
    subject: MoveSubject,
}

impl ReflogEntry {
    pub const FORMAT: &'static str = "--format=%H%x1f%cn <%ce> %ct%x1f%gn <%ge> %gd%x1f%gs";
    const SEPARATOR: char = '\u{1f}';

    pub fn created_since(&self, start: UnixSeconds) -> Option<&Commit> {
        (self.committed == self.moved && self.subject.creates() && self.committed.not_before(start))
            .then_some(&self.commit)
    }
}

impl FromStr for ReflogEntry {
    type Err = ();

    fn from_str(line: &str) -> Result<Self, ()> {
        let fields: Vec<&str> = line.split(Self::SEPARATOR).collect();
        match fields.as_slice() {
            [sha, committed, moved, subject] => Ok(Self {
                commit: sha.parse().map_err(drop)?,
                committed: committed.parse().map_err(drop)?,
                moved: moved.parse().map_err(drop)?,
                subject: subject.parse().map_err(drop)?,
            }),
            _ => Err(()),
        }
    }
}
