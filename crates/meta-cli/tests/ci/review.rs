use std::path::Path;

use super::{BLOCK, FAILURE, Git, NEXT_COMMIT, POST, PRE, PUSH_WORDS, Planted, Repo, STOP, Session, StartFile};

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
const SPOOF: [&str; 5] = ["commit", "-q", "--allow-empty", "-m", "Fast-forward (abort) (start) (finish)"];
const OLD_AUTHOR: [&str; 6] = ["commit", "-q", "--allow-empty", "--date=@1000000000 +0000", "-m", "dated"];
const AUTHOR_DATES: [&str; 5] = ["rebase", "-q", "--force-rebase", "--committer-date-is-author-date", "HEAD~1"];
const TO_MAIN: [&str; 4] = ["push", "-q", "origin", "feature:main"];
const SINGLE: [&str; 4] = ["--single-branch", "-b", "main", "-q"];
const NO_REFLOG: [&str; 4] = ["-c", "core.logAllRefUpdates=false", "-b", "feature"];
const BRANCH: [&str; 4] = ["checkout", "-q", "-b", "feature2"];
const PUSH_BRANCH: [&str; 5] = ["push", "-q", "-u", "origin", "feature2"];
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

#[test]
fn commit_text_commit_dates_and_a_cd_first_never_hide_a_commit() {
    let repo = Repo::new(FAILURE);
    let clone: &Path = repo.clone.path();
    let [away, cd, text, dated] = [(repo.sessions.path(), CD), (clone, CD), (clone, TEXT), (clone, DATED)]
        .map(|(cwd, id)| Session { repo: &repo, cwd, id });
    away.hook(PRE, CD_FIRST);
    let started = StartFile::read(repo.sessions.path(), CD).unwrap_or_else(|| Git::LONG_AGO.to_owned());
    Git::run_at(clone, &NEXT_COMMIT, &started);
    Git::run(clone, &PUSH_WORDS);
    let first = cd.hook(STOP, READ);
    let _: Vec<()> = [(TEXT, TEXT_AT), (DATED, DATED_AT)]
        .iter()
        .map(|(id, at)| StartFile::plant(repo.sessions.path(), id, at))
        .collect();
    text.hook(PRE, WRITE);
    Git::run_at(clone, &SPOOF, TEXT_AT);
    Git::run(clone, &PUSH_WORDS);
    let spoofed = text.hook(STOP, READ);
    Git::run_at(clone, &OLD_AUTHOR, DATED_AT);
    Git::run_at(clone, &AUTHOR_DATES, DATED_AT);
    Git::run(clone, &PUSH_WORDS);
    assert!([first, spoofed, dated.hook(STOP, READ)].iter().all(|stop| stop.contains(BLOCK)));
}

#[test]
fn readers_and_single_branch_clones_are_judged_by_what_the_remote_holds() {
    let repo = Repo::new(FAILURE);
    let clone: &Path = repo.clone.path();
    Git::run(clone, &TO_MAIN);
    let single = Clones::made_with(&repo, &SINGLE);
    let [reader, worker, lone] =
        [(clone, READER), (clone, WORKER), (single.path(), LONE)].map(|(cwd, id)| Session { repo: &repo, cwd, id });
    let _: Vec<String> =
        [(&reader, READ), (&worker, WRITE), (&lone, WRITE)].iter().map(|(s, c)| s.hook(PRE, c)).collect();
    Git::run(clone, &NEXT_COMMIT);
    worker.hook(POST, WRITE);
    Git::run(clone, &NEXT_COMMIT);
    let read = reader.hook(STOP, READ);
    let _: Vec<_> =
        [BRANCH.as_slice(), &NEXT_COMMIT, &PUSH_BRANCH].iter().map(|words| Git::run(single.path(), words)).collect();
    let pushed = lone.hook(STOP, READ);
    Git::run(single.path(), &BACK_TO_MAIN);
    let stops = [pushed, lone.hook(STOP, READ)];
    assert!(!read.contains(BLOCK) && stops.iter().all(|stop| stop.contains(BLOCK) && !stop.contains(NOT_PUSHED)));
}

#[test]
fn a_reflog_that_is_off_or_an_unwritable_state_dir_fails_closed() {
    let repo = Repo::new(FAILURE);
    let clone: &Path = repo.clone.path();
    Git::run(clone, &PUSH_WORDS);
    let quiet = Clones::made_with(&repo, &NO_REFLOG);
    let [blind, locked] = [(quiet.path(), BLIND), (clone, LOCKED)].map(|(cwd, id)| Session { repo: &repo, cwd, id });
    blind.hook(PRE, WRITE);
    Git::run(quiet.path(), &NEXT_COMMIT);
    Git::run(quiet.path(), &PUSH_WORDS);
    let off = blind.hook(STOP, READ);
    std::fs::create_dir_all(clone.join(STATE_ROOT)).unwrap();
    let open = Planted::locked(&clone.join(STATE_ROOT));
    locked.hook(PRE, WRITE);
    Git::run(clone, &NEXT_COMMIT);
    let warned = locked.hook(POST, WRITE);
    let stop = locked.hook(STOP, READ);
    std::fs::set_permissions(clone.join(STATE_ROOT), open).unwrap();
    assert!(off.contains(REFLOG_OFF) && warned.contains(NOT_WATCHED) && stop.contains(BLOCK));
}
