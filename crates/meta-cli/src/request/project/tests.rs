use std::path::Path;
use std::process::Command;
use std::time::{Duration, UNIX_EPOCH};

use meta_run::Budget;

use super::Project;
use crate::request::ci::{Commit, StartMoment};

const GIT_PROGRAM: &str = "git";
const BRANCH: &str = "feature";
const CLONE: &str = "clone";
const IDENTITY: [&str; 4] = ["-c", "user.name=t", "-c", "user.email=t@meta.invalid"];
const COMMIT: [&str; 4] = ["commit", "--quiet", "--allow-empty", "--message=c"];
const SETUP: [&[&str]; 3] = [
    &["init", "--quiet", "--bare", "origin.git"],
    &["init", "--quiet", "--initial-branch", BRANCH, CLONE],
    &["-C", CLONE, "remote", "add", "origin", "../origin.git"],
];
const REPOINTED: [&str; 4] = ["remote", "set-url", "origin", "nowhere.git"];
const PROTECTING: &str = "remote = \"origin\"\nprotected = [\"main\"]\n";
const PATIENT: Duration = Duration::from_secs(30);
const EPOCH: &str = "0";
const COMMITS: usize = 2;
const PUSH_FAILED: &str = "push of feature failed";
const CONFIGS: [&str; 3] = ["[ci]\nstop_millis = 9000", "[ci]\ngit_millis = \"soon\"", "remote = \"origin\""];
const SHOWN: [&str; 3] = ["stop_millis must be under", "git_millis", "origin"];
const GIT: [&str; 3] = [".git/refs", ".git/objects/f2", ".git/HEAD"];
const SHAS: [&str; 2] = ["f2e83b47f04f1aadaaf72b9d15074e933a513dc6", "9271c82b76058cb9022078bf58c641cfc6a5b0d7"];

struct Git;

impl Git {
    fn ran(dir: &Path, words: &[&str]) -> bool {
        Command::new(GIT_PROGRAM).current_dir(dir).args(words).status().is_ok_and(|status| status.success())
    }
}

#[test]
fn a_clones_reflog_names_its_branch_and_new_commits_which_push_and_list_on_the_remote_and_a_failed_push_says_why() {
    let dir = tempfile::tempdir().unwrap();
    let clone = dir.path().join(CLONE);
    let committed = || Git::ran(&clone, &[IDENTITY.as_slice(), &COMMIT].concat());
    assert!(SETUP.iter().all(|words| Git::ran(dir.path(), words)) && committed() && committed());
    std::fs::write(clone.join(Project::CONFIG_FILE), PROTECTING).unwrap();
    let project = Project::try_from(clone.as_path()).unwrap();
    let deadline = project.ci_settings().remote_deadline();
    let created = project.created_after(&EPOCH.parse::<StartMoment>().unwrap(), deadline).unwrap_or_default();
    let head = project.head().unwrap();
    let unpushed = project.unpushed(std::slice::from_ref(&head), &[], deadline).unwrap_or_default();
    let tips = project.independent_tips(&created, deadline).unwrap_or_default();
    let named = project.checkout(deadline).branch().map(ToString::to_string);
    assert!(
        [created.len(), unpushed.len()] == [COMMITS; 2] && tips == [head.clone()] && named.as_deref() == Some(BRANCH)
    );
    let pushed = project.push_failure(Budget::new(PATIENT));
    let listed = project.remote_heads(deadline).unwrap().tips::<Commit>();
    assert!(Git::ran(&clone, &REPOINTED));
    let refused = project.push_failure(Budget::new(PATIENT)).map(|failure| failure.to_string()).unwrap_or_default();
    assert!(pushed.is_none() && listed == [head] && refused.starts_with(PUSH_FAILED), "{refused}");
}

#[test]
fn a_project_serves_its_meta_toml_and_reads_head_and_object_births_through_git_and_no_meta_toml_is_no_project() {
    let dir = tempfile::tempdir().unwrap();
    GIT[..2].iter().for_each(|made| std::fs::create_dir_all(dir.path().join(made)).unwrap());
    assert!(Project::found(dir.path()).is_ok_and(|project| project.is_none()));
    let shown = CONFIGS.map(|text| {
        std::fs::write(dir.path().join("meta.toml"), text).unwrap();
        Project::try_from(dir.path())
            .map_or_else(|failure| failure.to_string(), |project| format!("{:?}", project.config().remote()))
    });
    assert!(shown.iter().zip(SHOWN).all(|(shown, named)| shown.contains(named)), "{shown:?}");
    let [head, unborn] = SHAS.map(|sha| sha.parse::<Commit>().unwrap());
    let object = dir.path().join(GIT[0]).with_file_name(head.loose_object());
    [dir.path().join(GIT[2]), object.clone()].iter().try_for_each(|file| std::fs::write(file, SHAS[0])).unwrap();
    let modified = std::fs::metadata(object).and_then(|meta| meta.modified()).unwrap().duration_since(UNIX_EPOCH);
    let starts = [0, modified.unwrap().as_nanos()].map(|nanos| nanos.to_string().parse::<StartMoment>().unwrap());
    let project = Project::try_from(dir.path()).unwrap();
    let deadline = project.ci_settings().remote_deadline();
    let judged = starts.map(|start| [&head, &unborn].map(|commit| project.born_after(commit, &start, deadline)));
    assert!(judged == [[true, false], [false, false]] && project.head() == Some(head), "{judged:?}");
}
