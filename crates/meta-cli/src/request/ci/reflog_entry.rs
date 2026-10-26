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
    pub const FORMAT: &'static str = "--format=%H%x1f%cn <%ce>%x1f%ct%x1f%gn <%ge>%x1f%gd%x1f%gs";
    const SEPARATOR: char = '\u{1f}';

    pub fn created_since(&self, start: UnixSeconds) -> Option<&Commit> {
        let here = self.committed == self.moved && self.subject.creates();
        (here && self.committed.at().is_some_and(|at| at >= start)).then_some(&self.commit)
    }
}

impl FromStr for ReflogEntry {
    type Err = ();

    fn from_str(line: &str) -> Result<Self, ()> {
        let fields: Vec<&str> = line.split(Self::SEPARATOR).collect();
        let [sha, committer, committed_at, mover, selector, subject] = fields.as_slice() else { return Err(()) };
        let Ok(commit) = sha.parse();
        Ok(Self {
            commit,
            committed: Stamp::new(committer, committed_at),
            moved: Stamp::from_selector(mover, selector),
            subject: MoveSubject::new(subject),
        })
    }
}
