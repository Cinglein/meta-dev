#![allow(clippy::unwrap_used)]

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output};

const META: &str = env!("CARGO_BIN_EXE_meta");
const OWNER: &str = "owner";
const WRITER: &str = "writer";
const NOWHERE: [&str; 4] = ["remote", "set-url", "origin", "/nonexistent/origin.git"];
const COUNT: [&str; 3] = ["rev-list", "--count", "meta/log"];
const ISSUE: [&str; 4] = ["issue", "--division", "features-and-maintenance", "text"];
const OFFLINE: &str = "offline";
const WRITES: &str = "2";
const LIST_ISSUES: [&str; 2] = ["list", "issues"];
const LOCAL_NAME: &str = "localname";
const OPEN: [&str; 4] = ["open", "--division", "features-and-maintenance", "text"];
const SEED: [[&str; 5]; 3] = [
    ["commit", "-q", "--allow-empty", "-m", "seed"],
    ["push", "-q", "origin", "HEAD:main", "--no-verify"],
    ["remote", "set-head", "origin", "main", "--"],
];
const NOREPLY: &str = "1+agent@users.noreply.github.com";
const CLAIM: &str = "claim";
const WORKTREE: &str = "--worktree";
const WORKTREES: [&str; 1] = ["worktrees"];
const SHOWN_BY_GIT: [[&str; 2]; 4] =
    [["config", "author.email"], ["config", "committer.email"], ["config", "user.name"], ["branch", "--show-current"]];
const FOREIGN_IDENTITY: &str = "[author]\n\temail = someone@example.com\n[committer]\n\temail = someone@example.com\n";
const RECORD_DIR: &str = ".git/meta-local";
const RECORD: &str = ".git/meta-local/worktrees.jsonl";
const TORN: &str = "{\"change\":\"Crea";
const LINE_2: &str = "line 2";
const MISSING: &str = "missing";
const AGENT: &str = "agent";
const STRAY: [&str; 4] = ["worktree", "add", "-q", "--detach"];
const UNRECORDED: &str = "unrecorded";
const EXCLUDE: &str = ".git/info/exclude";
const LOCAL_SETTINGS_RULE: &str = "/meta.local.toml\n";
const NOT_A_DIRECTORY: &str = "/dev/null";
const IGNORED: &str = "not gitignored";
const GIT_CALL: &str = "git -C /";

struct Git;

impl Git {
    const IDENTITY: [&'static str; 6] =
        ["-c", "user.name=test", "-c", "user.email=test@meta.invalid", "-c", "init.defaultBranch=main"];

    fn run(dir: &Path, words: &[&str]) -> Output {
        Command::new("git").arg("-C").arg(dir).args(Self::IDENTITY).args(words).output().unwrap()
    }

    fn shown(dir: &Path, words: &[&str]) -> String {
        Run(Command::new("git").arg("-C").arg(dir).args(words).output().unwrap()).to_string()
    }

    fn link_origin(dir: &Path, url: &str) {
        let remote = format!("[remote \"origin\"]\n\turl = {url}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n");
        Self::appended(&dir.join(".git/config"), &remote);
    }

    fn appended(file: &Path, text: impl AsRef<[u8]>) {
        std::fs::OpenOptions::new().append(true).open(file).unwrap().write_all(text.as_ref()).unwrap();
    }
}

struct Settings;

impl Settings {
    fn written(clone: &Path, worktrees: &Path) {
        let root = worktrees.display();
        let settings = format!("worktree_root = \"{root}\"\n[identity]\nname = \"{AGENT}\"\nemail = \"{NOREPLY}\"\n");
        std::fs::write(clone.join("meta.local.toml"), settings).unwrap();
    }
}

struct Run(Output);

struct Complaint(String);

struct Written(String);

impl Run {
    const ACTOR_INDEX: usize = 2;
    const ID_INDEX: usize = 4;

    fn actor(&self) -> Option<String> {
        self.to_string().split_whitespace().nth(Self::ACTOR_INDEX).map(str::to_owned)
    }

    fn complaint(&self) -> Complaint {
        Complaint(String::from_utf8_lossy(&self.0.stderr).into_owned())
    }

    fn written(&self) -> Written {
        Written(self.to_string().split_whitespace().nth(Self::ID_INDEX).unwrap_or_default().to_owned())
    }
}

impl std::fmt::Display for Run {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&String::from_utf8_lossy(&self.0.stdout))
    }
}

struct Project {
    origin: tempfile::TempDir,
    clone: tempfile::TempDir,
}

impl Project {
    fn new() -> Self {
        let origin = tempfile::tempdir().unwrap();
        let clone = tempfile::tempdir().unwrap();
        Git::run(origin.path(), &["init", "-q", "--bare"]);
        Git::run(clone.path(), &["init", "-q"]);
        Git::link_origin(clone.path(), origin.path().to_str().unwrap());
        std::fs::write(clone.path().join("meta.toml"), "remote = \"origin\"\n").unwrap();
        Self { origin, clone }
    }

    fn seeded(worktrees: &Path) -> Self {
        let project = Self::new();
        assert!(SEED.iter().all(|words| Git::run(project.clone.path(), words).status.success()));
        std::fs::write(project.clone.path().join(EXCLUDE), LOCAL_SETTINGS_RULE).unwrap();
        Git::appended(&project.clone.path().join(".git/config"), FOREIGN_IDENTITY);
        Settings::written(project.clone.path(), worktrees);
        project
    }

    fn meta(&self, actor: &str, words: &[&str]) -> Run {
        Run(Command::new(META).current_dir(self.clone.path()).args(words).env("META_ACTOR", actor).output().unwrap())
    }

    fn meta_unset(&self, words: &[&str]) -> Run {
        let mut command = Command::new(META);
        command.current_dir(self.clone.path()).args(words).env_remove("META_ACTOR").env("USER", LOCAL_NAME);
        Run(command.output().unwrap())
    }
}

#[test]
fn writes_go_to_the_remote_as_meta_actor_else_owner_and_offline_writes_are_refused() {
    let project = Project::new();
    let opened = project.meta_unset(&ISSUE);
    let listed = project.meta(OWNER, &LIST_ISSUES).to_string();
    assert!(opened.actor().as_deref() == Some(OWNER) && listed.contains(&opened.written().0));
    assert!(Run(Git::run(project.origin.path(), &COUNT)).to_string().starts_with(WRITES));
    assert_eq!(project.meta(WRITER, &ISSUE).actor().as_deref(), Some(WRITER));
    Git::run(project.clone.path(), &NOWHERE);
    assert!(project.meta(OWNER, &ISSUE).complaint().0.contains(OFFLINE));
}

#[test]
fn claim_with_worktree_makes_a_noreply_worktree_on_the_task_branch_that_worktrees_lists() {
    let worktrees = tempfile::tempdir().unwrap();
    let project = Project::seeded(worktrees.path());
    let task = project.meta(OWNER, &OPEN).written().0;
    let claimed = project.meta(OWNER, &[CLAIM, &task, WORKTREE]).to_string();
    let path = Path::new(claimed.lines().last().unwrap_or_default());
    let shown = SHOWN_BY_GIT.map(|words| Git::shown(path, &words));
    assert_eq!(shown, [NOREPLY, NOREPLY, AGENT, &task].map(|line| format!("{line}\n")));
    let stray = worktrees.path().join(AGENT);
    Git::run(project.clone.path(), &[&STRAY[..], &[stray.to_str().unwrap()]].concat());
    Git::appended(&project.clone.path().join(RECORD), TORN);
    let run = project.meta(OWNER, &WORKTREES);
    let listed = run.to_string();
    assert!(listed.contains(&format!("{task} assigned {task} ")) && !listed.contains(MISSING));
    let named = format!("{UNRECORDED} {}\n", std::fs::canonicalize(stray).unwrap().display());
    assert!(listed.ends_with(&named) && listed.matches(UNRECORDED).count() == 1 && run.complaint().0.contains(LINE_2));
}

#[test]
fn failed_worktrees_leave_no_claim_and_a_held_task_gets_its_lost_worktree_back_without_a_new_claim() {
    let worktrees = tempfile::tempdir().unwrap();
    let project = Project::seeded(Path::new(NOT_A_DIRECTORY));
    let task = project.meta(OWNER, &OPEN).written().0;
    let claim = [CLAIM, task.as_str(), WORKTREE];
    let writes = || Git::shown(project.origin.path(), &COUNT);
    let opened = writes();
    std::fs::write(project.clone.path().join(EXCLUDE), "").unwrap();
    let tracked = project.meta(OWNER, &claim).complaint().0;
    std::fs::write(project.clone.path().join(EXCLUDE), LOCAL_SETTINGS_RULE).unwrap();
    let unwritable = project.meta(OWNER, &claim).complaint().0;
    Settings::written(project.clone.path(), worktrees.path());
    std::fs::write(project.clone.path().join(RECORD_DIR), "").unwrap();
    project.meta(OWNER, &claim);
    std::fs::remove_file(project.clone.path().join(RECORD_DIR)).unwrap();
    let failed = writes();
    let first = project.meta(OWNER, &claim).to_string().lines().last().unwrap_or_default().to_owned();
    let claimed = writes();
    std::fs::remove_dir_all(&first).ok();
    let resumed = project.meta(OWNER, &claim).to_string();
    assert!(failed == opened && claimed != opened && writes() == claimed);
    assert!(tracked.contains(IGNORED) && unwritable.contains(NOT_A_DIRECTORY) && unwritable.contains(GIT_CALL));
    assert!(resumed.ends_with(&format!("{first}\n")) && Path::new(&first).exists());
}
