use std::path::Path;

use super::{BLOCK, COMMIT, FAILURE, Git, NEXT_COMMIT, PUSHED, Repo, STOP, Session, StartFile};

const MERGER: &str = "MERGER7";
const MERGER_START: &str = "3999990000";
const MERGED_AT: &str = "3999990001";
const SIDE: [&str; 4] = ["checkout", "-q", "-b", "side"];
const BACK: [&str; 3] = ["checkout", "-q", "feature"];
const MERGE_SIDE: [&str; 6] = ["merge", "-q", "--no-ff", "side", "-m", "merged"];

#[test]
fn a_merge_made_here_counts_by_its_parents_not_its_message() {
    let repo = Repo::new(FAILURE);
    let clone: &Path = repo.clone.path();
    StartFile::plant(repo.sessions.path(), MERGER, MERGER_START);
    let _: Vec<_> = [SIDE.as_slice(), &NEXT_COMMIT, &BACK].iter().map(|words| Git::run(clone, words)).collect();
    Git::run_at(clone, &MERGE_SIDE, MERGED_AT);
    Git::pushed(&repo, PUSHED);
    assert!(Session { repo: &repo, cwd: clone, id: MERGER }.hook(STOP, COMMIT).contains(BLOCK));
}
