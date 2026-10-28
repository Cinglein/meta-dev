#![allow(clippy::unwrap_used)]

use std::fmt;
use std::io::Write as _;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

const META: &str = env!("CARGO_BIN_EXE_meta");
const CONFIG: &str = "remote = \"origin\"\nprotected = [\"main\"]\n";
const MERGE: &str = "gh pr merge 1";
const PUSH: &str = "git push --force origin feature";
const ALLOWED: &str = "git push -u origin feature";
const DENY: &str = "\"hookEventName\":\"PreToolUse\",\"permissionDecision\":\"deny\",\"permissionDecisionReason\":\"meta-development: ";
const PUSH_REASON: &str = "meta-development: git push";
const PRE: &str = "pre-tool-use";
const POST: &str = "post-tool-use";
const COMMIT: &str = "git commit -m change";
const STATUS: &str = "git status";
const ALL_REFS: &str = "--all";
const AMEND: [&str; 6] = ["commit", "-q", "--amend", "--allow-empty", "-m", "rewritten"];
const FOREIGN_IDENTITY: &str = "[user]\n\tname = someone\n\temail = someone@example.com\n";
const NOREPLY_IDENTITY: &str = "[user]\n\tname = someone\n\temail = 1+someone@users.noreply.github.com\n";
const GIT_CONFIG: &str = ".git/config";
const BARE_CONFIG: &str = "config";
const ELSEWHERE: &str = "ELSEWHERE";
const FOREIGN_REASON: &str = "someone@example.com is not noreply; git config user.name";
const UNRESOLVED_REASON: &str = "cannot resolve the commit directory";
const REFUSED_COMMITS: [(&str, &str); 4] = [
    ("cd ELSEWHERE && git commit --amend --no-edit", FOREIGN_REASON),
    ("git commit -m change", FOREIGN_REASON),
    ("git commit -m x && git -C ELSEWHERE commit -m y", FOREIGN_REASON),
    ("git -C \"$REPO\" commit -m x", UNRESOLVED_REASON),
];
const NEW_FEATURE: [&str; 4] = ["switch", "-q", "-c", "feature"];
const HEAD: &str = "HEAD";
const FEATURE_BRANCH: &str = "feature";
const REJECTED: &str = "rejected";
const SESSION_START: &str = "session-start";
const PATIENT: Duration = Duration::from_secs(2);
const POLL: Duration = Duration::from_millis(2);
const ENV_FILE: &str = ".env";
const SESSION_FIELD: &str = r#""session_id":"ABCDEF12-34","#;
const WATCHERS: &str = ".git/meta-ci";
const NO_TASKS: &str = "Tasks: 0 unfinished";
const SESSION_ACTOR: &str = "claude-abcdef1234";
const SESSION_STARTED: &str = "meta-sessions/ABCDEF1234/started";
const ENV_LINE: &str = "export META_ACTOR=claude-abcdef1234\nexport META_SESSION=ABCDEF12-34\n";
const HANGING_UPLOAD: &str = "[remote \"origin\"]\n\tuploadpack = sh -c 'sleep 5' x\n";
const META_TOML: &str = "meta.toml";
const QUICK_GIT: &str = "[ci]\ngit_millis = 20\n";
const UNREADABLE: &str = "meta/log unreadable";

#[path = "support/skeleton.rs"]
mod skeleton;
use skeleton::Skeleton;

struct Git;

impl Git {
    const IDENTITY: [&'static str; 8] = [
        "-c",
        "user.name=t",
        "-c",
        "user.email=t@meta.invalid",
        "-c",
        "init.defaultBranch=main",
        "-c",
        "maintenance.auto=false",
    ];

    fn run(dir: &Path, words: &[&str]) -> Output {
        Command::new("git").arg("-C").arg(dir).args(Self::IDENTITY).args(words).output().unwrap()
    }

    fn link_origin(dir: &Path, origin: &Path) {
        let url = origin.display();
        Self::configure(
            &dir.join(".git/config"),
            &format!("[remote \"origin\"]\n\turl = {url}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n"),
        );
    }

    fn configure(file: &Path, section: &str) {
        std::fs::OpenOptions::new().append(true).open(file).unwrap().write_all(section.as_bytes()).unwrap();
    }

    fn head(dir: &Path, reference: &str) -> Revision {
        Revision(String::from_utf8_lossy(&Self::run(dir, &["rev-parse", reference]).stdout).into_owned())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Revision(String);

struct Repo {
    origin: tempfile::TempDir,
    clone: tempfile::TempDir,
}

impl Repo {
    fn new() -> Self {
        let origin = tempfile::tempdir().unwrap();
        let clone = tempfile::tempdir().unwrap();
        Skeleton::Bare.lay(origin.path(), "main");
        Skeleton::Working.lay(clone.path(), "main");
        Git::link_origin(clone.path(), origin.path());
        std::fs::write(clone.path().join("meta.toml"), CONFIG).unwrap();
        Git::run(clone.path(), &["commit", "-q", "--allow-empty", "-m", "start"]);
        Self { origin, clone }
    }

    fn hook(&self, event: &str, command: &str, cwd: &Path) -> String {
        let child = self.started(event, command, cwd, &mut Command::new(META));
        String::from_utf8_lossy(&child.wait_with_output().unwrap().stdout).into_owned()
    }

    fn started(&self, event: &str, command: &str, cwd: &Path, meta: &mut Command) -> Child {
        let session = if event == SESSION_START { SESSION_FIELD } else { "" };
        let input = format!(
            r#"{{{session}"cwd":{},"tool_input":{{"command":{}}}}}"#,
            serde_json::to_string(cwd).unwrap(),
            serde_json::to_string(&command.replace(ELSEWHERE, &self.origin.path().display().to_string())).unwrap()
        );
        let meta = meta
            .args(["hook", event])
            .env_remove("META_ACTOR")
            .env("CLAUDE_ENV_FILE", self.clone.path().join(ENV_FILE));
        let piped = meta.env("TMPDIR", self.clone.path()).stdin(Stdio::piped()).stdout(Stdio::piped());
        let mut child = piped.spawn().unwrap();
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        child
    }

    fn interrupted(&self, helper: &Path) -> bool {
        let hanging =
            format!("[remote \"origin\"]\n\treceivepack = bash -c 'echo $$ > {} && sleep 5' x\n", helper.display());
        Git::configure(&self.clone.path().join(GIT_CONFIG), &hanging);
        let mut meta = self.started(POST, COMMIT, self.clone.path(), Command::new(META).process_group(0));
        let polls = || (0..PATIENT.as_millis() / POLL.as_millis()).map(|_| std::thread::sleep(POLL));
        let pid = polls().find_map(|()| std::fs::read_to_string(helper).ok().filter(|pid| !pid.is_empty()));
        Command::new("kill").args(["-INT", "--", &format!("-{}", meta.id())]).status().unwrap();
        let alive = |pid: &str| Command::new("kill").args(["-0", pid.trim()]).status().unwrap().success();
        meta.wait().is_ok() && pid.is_some_and(|pid| polls().any(|()| !alive(&pid)))
    }
}

#[test]
fn pre_tool_use_denies_merges_force_pushes_and_commits_by_foreign_emails_wherever_they_run() {
    let repo = Repo::new();
    let home = repo.clone.path();
    let elsewhere = repo.origin.path();
    Git::configure(&home.join(GIT_CONFIG), NOREPLY_IDENTITY);
    Git::configure(&elsewhere.join(BARE_CONFIG), FOREIGN_IDENTITY);
    let push = repo.hook(PRE, PUSH, home);
    assert!(repo.hook(PRE, MERGE, home).contains(DENY) && push.contains(DENY) && push.contains(PUSH_REASON));
    let commits = REFUSED_COMMITS.into_iter().zip([home, elsewhere, home, home]);
    let shown = commits.map(|((command, reason), cwd)| (repo.hook(PRE, command, cwd), format!("{DENY}{reason}")));
    assert!(shown.into_iter().all(|(shown, refused)| shown.contains(&refused)));
    assert_eq!(repo.hook(PRE, ALLOWED, home), "");
}

#[test]
fn post_tool_use_pushes_unprotected_branches_and_reports_git_errors() {
    let repo = Repo::new();
    let home = repo.clone.path();
    repo.hook(POST, COMMIT, home);
    Git::run(repo.clone.path(), &NEW_FEATURE);
    repo.hook(POST, STATUS, home);
    assert_eq!(Git::head(repo.origin.path(), ALL_REFS), Revision(String::new()));
    repo.hook(POST, COMMIT, home);
    assert_eq!(Git::head(repo.origin.path(), FEATURE_BRANCH), Git::head(repo.clone.path(), HEAD));
    Git::run(repo.clone.path(), &AMEND);
    assert!(repo.hook(POST, COMMIT, home).contains(REJECTED) && !repo.clone.path().join(WATCHERS).exists());
    let helper = tempfile::NamedTempFile::new().unwrap();
    assert!(repo.interrupted(helper.path()), "an interrupted hook left its git helper running");
}

#[test]
fn session_start_shows_state_and_names_the_session_actor() {
    let repo = Repo::new();
    let home = repo.clone.path();
    let shown = repo.hook(SESSION_START, COMMIT, home);
    assert!(shown.contains(NO_TASKS) && shown.contains(SESSION_ACTOR));
    assert_eq!(std::fs::read_to_string(home.join(ENV_FILE)).unwrap(), ENV_LINE);
    Git::configure(&home.join(GIT_CONFIG), HANGING_UPLOAD);
    Git::configure(&home.join(META_TOML), QUICK_GIT);
    let started = Instant::now();
    let shown = repo.hook(SESSION_START, COMMIT, home);
    assert!(home.join(SESSION_STARTED).exists() && started.elapsed() < PATIENT && shown.contains(UNREADABLE));
}

impl fmt::Display for Repo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.clone.path().display())
    }
}
