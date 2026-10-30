#![allow(clippy::unwrap_used)]

use std::fmt::Write as _;
use std::path::Path;

use meta_git::{
    FileCommit, Files, HeadReflog, Identity, IndependentTips, Layout, LooseObject, ObjectId, Reads, RefMove,
    Repository, Unpushed, Writes,
};

const NAME: &str = "agent";
const EMAIL: &str = "1+agent@users.noreply.github.com";
const REFS: [&str; 2] = ["refs/heads/main", "refs/remotes/origin/main"];
const FILES: [&str; 3] = ["first", "second", "side"];
const REMOTE: &str = "origin";
const SEPARATOR: &str = "\u{1f}";
const MOVED: &str = " HEAD@{";
const REASON: &str = "meta";

struct History {
    repository: Repository,
    first: ObjectId,
    second: ObjectId,
    side: ObjectId,
}

impl History {
    fn made(root: &Path) -> History {
        let repository = Repository::created(root, Layout::Working).unwrap();
        let commit = |parent: Option<ObjectId>, file: &str| {
            let files: Files = [(file.to_owned(), Some(file.to_owned()))].into_iter().collect();
            let author = Identity::new(NAME.to_owned(), EMAIL.to_owned());
            repository.write(&FileCommit::new(parent, files, author, file.to_owned())).unwrap()
        };
        let [first, second, side] = FILES;
        let first = commit(None, first);
        let second = commit(Some(first), second);
        let side = commit(Some(first), side);
        let moved = |name: &str, from, to| repository.write(&RefMove::new(name.parse().unwrap(), from, to)).unwrap();
        let [main, tracking] = REFS;
        moved(main, None, first);
        moved(main, Some(first), second);
        moved(tracking, None, first);
        History { repository, first, second, side }
    }
}

#[test]
fn the_head_reflog_reads_like_git_log_walk_reflogs_newest_first() {
    let root = tempfile::tempdir().unwrap();
    let History { repository, first, second, .. } = History::made(root.path());
    let shown = repository.read(&HeadReflog).unwrap().iter().fold(String::new(), |mut shown, line| {
        writeln!(shown, "{line}").unwrap();
        shown
    });
    let identity = [NAME, " <", EMAIL, ">"].concat();
    let read: Vec<_> = shown
        .lines()
        .map(|line| match line.split(SEPARATOR).collect::<Vec<_>>().as_slice() {
            [commit, parents, committer, moved, _, subject] => Some((
                commit.parse::<ObjectId>().ok(),
                parents.parse::<ObjectId>().ok(),
                *committer == identity && moved.contains(MOVED) && *subject == REASON,
            )),
            _ => None,
        })
        .collect();
    assert_eq!(read, [Some((Some(second), Some(first), true)), Some((Some(first), None, true))]);
}

#[test]
fn unpushed_commits_skip_the_remote_and_held_ones_and_only_independent_tips_remain() {
    let root = tempfile::tempdir().unwrap();
    let History { repository, first, second, side } = History::made(root.path());
    let unpushed = |held: Vec<ObjectId>| {
        let mut found = repository.read(&Unpushed::new(vec![second, side], REMOTE.parse().unwrap(), held)).unwrap();
        found.sort();
        found
    };
    let mut both = vec![second, side];
    both.sort();
    assert!(unpushed(vec![]) == both && unpushed(vec![side]) == [second]);
    let tips = repository.read(&IndependentTips::from(vec![first, second, side])).unwrap();
    let object = repository.read(&LooseObject::from(second)).unwrap();
    assert!(tips == [second, side] && object.exists());
}

#[test]
fn a_reflog_entry_whose_commit_is_gone_fails_the_whole_read() {
    let root = tempfile::tempdir().unwrap();
    let History { repository, second, .. } = History::made(root.path());
    std::fs::remove_file(repository.read(&LooseObject::from(second)).unwrap()).unwrap();
    assert!(Repository::open(root.path()).unwrap().read(&HeadReflog).is_err());
}
