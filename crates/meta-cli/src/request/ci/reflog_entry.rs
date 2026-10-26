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
    type Err = std::num::ParseIntError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut fields = line.split(Self::SEPARATOR);
        let Ok(commit) = fields.next().unwrap_or_default().parse();
        let committed = fields.next().unwrap_or_default().parse()?;
        let moved = fields.next().unwrap_or_default().parse()?;
        let Ok(subject) = fields.next().unwrap_or_default().parse();
        Ok(Self { commit, committed, moved, subject })
    }
}
