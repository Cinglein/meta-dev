mod identity;
mod move_subject;
mod stamp;

use std::str::FromStr;

use meta_git::ReflogLine;

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
    const SEPARATOR: char = '\u{1f}';

    pub fn created(entries: &[Self], start: &StartMoment, born_after: impl Fn(&Commit) -> bool) -> Vec<Commit> {
        let befores = entries.iter().skip(1).map(Some).chain([None]);
        let created: Vec<&Commit> = entries
            .iter()
            .zip(befores)
            .filter_map(|(entry, before)| entry.created_after(start, before, |commit| born_after(commit)))
            .collect();
        let first = |at: usize, commit: &Commit| created.iter().position(|other| *other == commit) == Some(at);
        created
            .iter()
            .enumerate()
            .filter(|(at, commit)| first(*at, commit))
            .map(|(_, commit)| (*commit).clone())
            .collect()
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

    pub fn read(lines: &[ReflogLine]) -> Result<Vec<Self>, ()> {
        lines.iter().map(|line| line.to_string().parse()).collect()
    }
}

impl FromStr for ReflogEntry {
    type Err = ();

    fn from_str(line: &str) -> Result<Self, ()> {
        let fields: Vec<&str> = line.split(Self::SEPARATOR).collect();
        match fields.as_slice() {
            [sha, parents, committer, moved, _, subject] => {
                let commit: Commit = sha.parse().map_err(drop)?;
                Ok(Self {
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

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::SystemTime;

    use meta_git::{
        FileCommit, Files, HeadReflog, Identity, Layout, ObjectId, Reads as _, RefMove, Repository, Writes as _,
    };

    use super::ReflogEntry;

    const AUTHOR: [&str; 2] = ["agent", "1+agent@users.noreply.github.com"];
    const MOVER: &str = "[user]\n\tname = mover\n\temail = 2+mover@users.noreply.github.com\n";
    const MOVER_SHOWN: &str = "mover <2+mover@users.noreply.github.com>";
    const MAIN: &str = "refs/heads/main";
    const SUBJECT: &str = "meta";
    const FILES: [&str; 2] = ["first", "second"];

    struct Moved {
        repository: Repository,
        commits: Vec<ObjectId>,
    }

    impl Moved {
        fn made(dir: &Path) -> Moved {
            let repository = Repository::created(dir, Layout::Working).unwrap();
            std::fs::write(dir.join(".git/config"), MOVER).unwrap();
            let [name, email] = AUTHOR;
            let commits = FILES.iter().fold(Vec::new(), |mut commits: Vec<ObjectId>, file| {
                let files: Files = [((*file).to_owned(), Some((*file).to_owned()))].into_iter().collect();
                let author = Identity::new(name.to_owned(), email.to_owned());
                let parent = commits.last().copied();
                let made = repository.write(&FileCommit::new(parent, files, author, (*file).to_owned())).unwrap();
                repository.write(&RefMove::new(MAIN.parse().unwrap(), parent, made)).unwrap();
                commits.push(made);
                commits
            });
            Moved { repository, commits }
        }

        fn now() -> u64 {
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
        }

        fn expected(&self, at: u64) -> Vec<ReflogEntry> {
            let [name, email] = AUTHOR;
            let line = |made: usize, commit: &ObjectId| {
                let parents: Vec<String> =
                    self.commits[made.saturating_sub(1)..made].iter().map(ToString::to_string).collect();
                let parents = parents.join(" ");
                let shown = format!(
                    "{commit}\u{1f}{parents}\u{1f}{name} <{email}>\u{1f}{MOVER_SHOWN} HEAD@{{{at}}}\u{1f}\u{1f}"
                );
                [shown, SUBJECT.to_owned()].concat().parse::<ReflogEntry>().unwrap()
            };
            self.commits.iter().enumerate().rev().map(|(made, commit)| line(made, commit)).collect()
        }
    }

    #[test]
    fn a_real_reflog_round_trips_into_exact_entries_with_mover_moment_parents_and_subject() {
        let (dir, before) = (tempfile::tempdir().unwrap(), Moved::now());
        let moved = Moved::made(dir.path());
        let read = ReflogEntry::read(&moved.repository.read(&HeadReflog).unwrap());
        assert!((before..=Moved::now()).any(|at| read == Ok(moved.expected(at))), "{read:?}");
    }
}
