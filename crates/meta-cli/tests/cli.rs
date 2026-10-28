#![allow(clippy::unwrap_used)]

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output, Stdio};

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
const SEED: &str = "commit refs/heads/main\ncommitter t <t@meta.invalid> 1700000000 +0000\ndata 4\nseed\n\n";
const TRACKED: &str = "reset refs/remotes/origin/main\nfrom refs/heads/main\n\n";
const OPENED: &str = "TaskOpened";
const ORIGIN_HEAD: [&str; 3] = ["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/main"];
const NOREPLY: &str = "1+agent@users.noreply.github.com";
const CLAIM: &str = "claim";
const WORKTREE: &str = "--worktree";
const WORKTREES: [&str; 1] = ["worktrees"];
const SHOWN_BY_GIT: [[&str; 2]; 4] =
    [["config", "author.email"], ["config", "committer.email"], ["config", "user.name"], ["branch", "--show-current"]];
const FOREIGN_IDENTITY: &str = "[author]\n\temail = someone@example.com\n[committer]\n\temail = someone@example.com\n";
const RECORD_DIR: &str = ".git/meta-local";
const AGENT: &str = "agent";
const STRAY: [&str; 4] = ["worktree", "add", "-q", "--detach"];
const SEEDED: &str = "1\n";
const CLAIMED: &str = "TaskClaimed";
const UNRECORDED: &str = "unrecorded";
const EXCLUDE: &str = ".git/info/exclude";
const LOCAL_SETTINGS_RULE: &str = "/meta.local.toml\n";
const TRACE: &str = "GIT_TRACE2_EVENT";
const STARTED: &str = "\"event\":\"start\"";
const LISTING: &str = "\"hierarchy\":\"ls-remote\"";
const READ_PROCESSES: [&str; 3] = ["ls-remote", "upload-pack", "cat-file"];

#[path = "support/seeded_log.rs"]
mod seeded_log;
#[path = "support/skeleton.rs"]
mod skeleton;
use seeded_log::{SeededLog, TASK};
use skeleton::Skeleton;

struct Git;

impl Git {
    const IDENTITY: [&'static str; 8] = [
        "-c",
        "user.name=test",
        "-c",
        "user.email=test@meta.invalid",
        "-c",
        "init.defaultBranch=main",
        "-c",
        "maintenance.auto=false",
    ];

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

    fn appended(file: &Path, text: impl std::fmt::Display) {
        write!(std::fs::OpenOptions::new().append(true).open(file).unwrap(), "{text}").unwrap();
    }
}

struct Seed;

impl Seed {
    fn imported(repo: &Path, stream: &str) {
        let mut command = Command::new("git");
        let piped = command.arg("-C").arg(repo).args(["fast-import", "--quiet"]).stdin(Stdio::piped());
        let mut child = piped.spawn().unwrap();
        child.stdin.take().unwrap().write_all(stream.as_bytes()).unwrap();
        assert!(child.wait().unwrap().success());
    }

    fn planted(origin: &Path, clone: &Path) {
        std::thread::scope(|scope| {
            scope.spawn(|| {
                Self::imported(origin, SEED);
                SeededLog::through(OPENED).plant(origin);
            });
            Self::imported(clone, &[SEED, TRACKED].concat());
            assert!(Git::run(clone, &ORIGIN_HEAD).status.success());
        });
    }
}

struct Run(Output);

struct Complaint(String);

struct Written(String);

struct Traced(Run, String);

impl Traced {
    fn run(project: &Project, words: &[&str]) -> Self {
        let trace = tempfile::NamedTempFile::new().unwrap();
        let mut command = Command::new(META);
        command.current_dir(project.clone.path()).args(words).env_remove("META_ACTOR").env("USER", LOCAL_NAME);
        command.env(TRACE, trace.path());
        Self(Run(command.output().unwrap()), std::fs::read_to_string(trace.path()).unwrap())
    }
}

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
        Skeleton::Bare.lay(origin.path(), "main");
        Skeleton::Working.lay(clone.path(), "main");
        Git::link_origin(clone.path(), origin.path().to_str().unwrap());
        std::fs::write(clone.path().join("meta.toml"), "remote = \"origin\"\n").unwrap();
        Self { origin, clone }
    }

    fn seeded(worktrees: &Path) -> Self {
        let project = Self::new();
        Seed::planted(project.origin.path(), project.clone.path());
        std::fs::create_dir(project.clone.path().join(".git/info")).unwrap();
        std::fs::write(project.clone.path().join(EXCLUDE), LOCAL_SETTINGS_RULE).unwrap();
        Git::appended(&project.clone.path().join(".git/config"), FOREIGN_IDENTITY);
        let root = worktrees.display();
        let settings = format!("worktree_root = \"{root}\"\n[identity]\nname = \"{AGENT}\"\nemail = \"{NOREPLY}\"\n");
        std::fs::write(project.clone.path().join("meta.local.toml"), settings).unwrap();
        project
    }

    fn meta(&self, actor: &str, words: &[&str]) -> Run {
        Run(Command::new(META).current_dir(self.clone.path()).args(words).env("META_ACTOR", actor).output().unwrap())
    }
}

#[test]
fn writes_go_to_the_remote_as_meta_actor_else_owner_listing_it_once_and_offline_writes_are_refused() {
    let project = Project::new();
    let opened = Traced::run(&project, &ISSUE);
    let listed = Traced::run(&project, &LIST_ISSUES);
    let started = listed.1.matches(STARTED).count();
    assert!(opened.0.actor().as_deref() == Some(OWNER) && listed.0.to_string().contains(&opened.0.written().0));
    assert!(opened.1.matches(LISTING).count() == 1 && (1..=READ_PROCESSES.len()).contains(&started), "{started}");
    assert!(Run(Git::run(project.origin.path(), &COUNT)).to_string().starts_with(WRITES));
    assert_eq!(project.meta(WRITER, &ISSUE).actor().as_deref(), Some(WRITER));
    Git::run(project.clone.path(), &NOWHERE);
    assert!(project.meta(OWNER, &ISSUE).complaint().0.contains(OFFLINE));
}

#[test]
fn a_claim_failing_after_its_worktree_leaves_no_claim_branch_or_path_so_a_retry_claims_a_listed_noreply_worktree() {
    let worktrees = tempfile::tempdir().unwrap();
    let project = Project::seeded(worktrees.path());
    let claim = || project.meta(OWNER, &[CLAIM, TASK, WORKTREE]);
    std::fs::write(project.clone.path().join(RECORD_DIR), "").unwrap();
    let failed = claim().complaint().0;
    std::fs::remove_file(project.clone.path().join(RECORD_DIR)).unwrap();
    assert!(failed.contains(RECORD_DIR) && Git::shown(project.origin.path(), &COUNT) == SEEDED);
    let claimed = claim().to_string();
    let shown = SHOWN_BY_GIT.map(|words| Git::shown(Path::new(claimed.lines().last().unwrap_or_default()), &words));
    assert!(claimed.contains(CLAIMED) && shown == [NOREPLY, NOREPLY, AGENT, TASK].map(|line| format!("{line}\n")));
    let stray = worktrees.path().join(AGENT);
    Git::run(project.clone.path(), &[&STRAY[..], &[stray.to_str().unwrap()]].concat());
    let listed = project.meta(OWNER, &WORKTREES).to_string();
    let named = format!("{UNRECORDED} {}\n", std::fs::canonicalize(stray).unwrap().display());
    assert!(listed.contains(&format!("{TASK} assigned {TASK} ")) && listed.ends_with(&named));
}
