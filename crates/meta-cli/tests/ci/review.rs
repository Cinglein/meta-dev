use std::path::Path;

use super::scenes::Scene;
use super::{
    BLOCK, FAILURE, FailingHead, Git, NEXT_COMMIT, POST, PRE, PUSHED_BRANCH, Planted, Remote, Repo, STOP, Session,
    StartFile,
};

const WRITE: &str = "git commit -m x";
const READ: &str = "cargo test";
const CD_FIRST: &str = "cd c && git commit -m x && git push";
const CD: &str = "CDFIRST1";
const TEXT: &str = "TEXT2";
const DATED: &str = "DATED3";
const READER: &str = "READER4";
const WORKER: &str = "WORKER5";
const LONE: &str = "LONE6";
const BLIND: &str = "BLIND7";
const LOCKED: &str = "LOCKED8";
const TEXT_AT: &str = "4000000000";
const DATED_AT: &str = "4000000100";
const CD_AT: &str = "3999999000";
const LATER: &str = "3999999999";
const TEXT_COMMIT_AT: &str = "4000000001";
const DATED_COMMIT_AT: &str = "4000000101";
const SPOOF: [&str; 5] = ["commit", "-q", "--allow-empty", "-m", "Fast-forward (abort) (start) (finish)"];
const OLD_AUTHOR: [&str; 6] = ["commit", "-q", "--allow-empty", "--date=@1000000000 +0000", "-m", "dated"];
const AUTHOR_DATES: [&str; 5] = ["rebase", "-q", "--force-rebase", "--committer-date-is-author-date", "HEAD~1"];
const MAIN: &str = "main";
const SINGLE: [&str; 4] = ["--single-branch", "-b", "main", "-q"];
const NO_REFLOG: [&str; 4] = ["-c", "core.logAllRefUpdates=false", "-b", "feature"];
const BRANCH: [&str; 4] = ["checkout", "-q", "-b", "feature2"];
const PUSH_BRANCH: [&str; 5] = ["push", "-q", "-u", "origin", "feature2"];
const PUSH_ASIDE: [&str; 4] = ["push", "-q", "origin", "HEAD:aside"];
const BACK_TO_MAIN: [&str; 3] = ["checkout", "-q", "main"];
const NOT_PUSHED: &str = "is not pushed";
const REFLOG_OFF: &str = "reflog";
const NOT_WATCHED: &str = "CI not watched";
const STATE_ROOT: &str = ".git/meta-ci";
const CONFIG_FILE: &str = "meta.toml";

struct Clones;

impl Clones {
    fn made_with(repo: &Repo, options: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let origin = repo.origin.path().to_str().unwrap();
        Git::run(dir.path(), &[["clone"].as_slice(), options, &[origin, "."]].concat());
        std::fs::copy(repo.clone.path().join(CONFIG_FILE), dir.path().join(CONFIG_FILE)).unwrap();
        dir
    }
}

pub struct CommitTextAndDates;

impl Scene for CommitTextAndDates {
    fn hold() {
        let repo = Repo::new(FAILURE);
        let clone: &Path = repo.clone.path();
        let [away, cd, text, dated] = [(repo.sessions.path(), CD), (clone, CD), (clone, TEXT), (clone, DATED)]
            .map(|(cwd, id)| Session { repo: &repo, cwd, id });
        away.hook(PRE, CD_FIRST);

        Git::run_at(clone, &NEXT_COMMIT, CD_AT);
        Remote::pushed(&repo, PUSHED_BRANCH);
        let first = cd.hook(STOP, READ);
        let _: Vec<()> = [(TEXT, TEXT_AT), (DATED, DATED_AT)]
            .iter()
            .map(|(id, at)| StartFile::plant(repo.sessions.path(), id, at))
            .collect();
        text.hook(PRE, WRITE);
        Git::run_at(clone, &SPOOF, TEXT_COMMIT_AT);
        Remote::pushed(&repo, PUSHED_BRANCH);
        let spoofed = text.hook(STOP, READ);
        Git::run_at(clone, &OLD_AUTHOR, DATED_COMMIT_AT);
        Git::run_at(clone, &AUTHOR_DATES, DATED_COMMIT_AT);
        Remote::pushed(&repo, PUSHED_BRANCH);
        assert!([first, spoofed, dated.hook(STOP, READ)].iter().all(|stop| stop.contains(BLOCK)));
    }
}

pub struct ReadersAndSingleBranchClones;

impl Scene for ReadersAndSingleBranchClones {
    fn hold() {
        let repo = Repo::new(FAILURE);
        let clone: &Path = repo.clone.path();
        Remote::pushed(&repo, MAIN);
        let single = Clones::made_with(&repo, &SINGLE);
        let [reader, worker, lone] =
            [(clone, READER), (clone, WORKER), (single.path(), LONE)].map(|(cwd, id)| Session { repo: &repo, cwd, id });
        let _: Vec<String> =
            [(&reader, READ), (&worker, WRITE), (&lone, WRITE)].iter().map(|(s, c)| s.hook(PRE, c)).collect();
        Git::run(clone, &NEXT_COMMIT);
        worker.hook(POST, WRITE);
        Git::run(clone, &NEXT_COMMIT);
        let read = reader.hook(STOP, READ);
        let _: Vec<_> = [BRANCH.as_slice(), &NEXT_COMMIT, &PUSH_BRANCH]
            .iter()
            .map(|words| Git::run_at(single.path(), words, LATER))
            .collect();
        FailingHead::copy_from(&repo, single.path());
        let pushed = lone.hook(STOP, READ);
        Git::run(single.path(), &BACK_TO_MAIN);
        let stops = [pushed, lone.hook(STOP, READ)];
        assert!(!read.contains(BLOCK) && stops.iter().all(|stop| stop.contains(BLOCK) && !stop.contains(NOT_PUSHED)));
    }
}

pub struct ReflogOffOrStateLocked;

impl Scene for ReflogOffOrStateLocked {
    fn hold() {
        let repo = Repo::new(FAILURE);
        let clone: &Path = repo.clone.path();
        Remote::pushed(&repo, PUSHED_BRANCH);
        let quiet = Clones::made_with(&repo, &NO_REFLOG);
        let [blind, locked] =
            [(quiet.path(), BLIND), (clone, LOCKED)].map(|(cwd, id)| Session { repo: &repo, cwd, id });
        blind.hook(PRE, WRITE);
        Git::run(quiet.path(), &NEXT_COMMIT);
        Git::run(quiet.path(), &PUSH_ASIDE);
        let off = blind.hook(STOP, READ);
        std::fs::create_dir_all(clone.join(STATE_ROOT)).unwrap();
        let open = Planted::locked(&clone.join(STATE_ROOT));
        locked.hook(PRE, WRITE);
        Git::run_at(clone, &NEXT_COMMIT, LATER);
        let warned = locked.hook(POST, WRITE);
        let stop = locked.hook(STOP, READ);
        std::fs::set_permissions(clone.join(STATE_ROOT), open).unwrap();
        assert!(off.contains(REFLOG_OFF) && warned.contains(NOT_WATCHED) && stop.contains(BLOCK));
    }
}
