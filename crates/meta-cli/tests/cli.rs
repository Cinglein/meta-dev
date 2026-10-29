#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::Output;

const OWNER: &str = "owner";
const WRITER: &str = "writer";
const NOWHERE: &str = "/nonexistent/origin.git";
const LOG: &str = "refs/heads/meta/log";
const ISSUE: [&str; 4] = ["issue", "--division", "features-and-maintenance", "text"];
const OFFLINE: &str = "offline";
const WRITES_LEN: usize = 2;
const LIST_ISSUES: [&str; 2] = ["list", "issues"];
const LOCAL_NAME: &str = "localname";
const SEED: &str = "seed";
const SEED_TIME: Moment = Moment::after_epoch(1_700_000_000);
const TRACKING: &str = "refs/remotes/origin/";
const OPENED: &str = "TaskOpened";
const ORIGIN_HEAD: [&str; 2] = [".git/refs/remotes/origin/HEAD", "ref: refs/remotes/origin/main\n"];
const NOREPLY: &str = "1+agent@users.noreply.github.com";
const CLAIM: &str = "claim";
const WORKTREE: &str = "--worktree";
const WORKTREES: [&str; 1] = ["worktrees"];
const SHOWN_BY_GIT: [&str; 3] = ["author.email", "committer.email", "user.name"];
const FOREIGN_IDENTITY: [&str; 2] = ["someone@example.com", "someone@example.com"];
const RECORD_DIR: &str = ".git/meta-local";
const AGENT: &str = "agent";
const SEEDED_LEN: usize = 1;
const CLAIMED: &str = "TaskClaimed";
const UNRECORDED: &str = "unrecorded";
const EXCLUDE: &str = ".git/info/exclude";
const LOCAL_SETTINGS_RULE: &str = "/meta.local.toml\n";
const TRACE: &str = "GIT_TRACE2_EVENT";
const STARTED: &str = "\"event\":\"start\"";
const LISTING: &str = "\"hierarchy\":\"ls-remote\"";
const READ_PROCESSES: [&str; 3] = ["ls-remote", "upload-pack", "cat-file"];
const TARGET: &str = "target.noindex";
const CARGO_TAG: [&str; 2] =
    ["CACHEDIR.TAG", "Signature: 8a477f597d28d172789f06886806bc55\n# This file is a cache directory tag created by cargo.\n"];
const PROFILES: [&str; 2] = ["debug", "release"];
const LOCK: &str = ".cargo-lock";
const ARTIFACT: [&str; 2] = ["deps/libx.rlib", "compiled"];

#[allow(dead_code)]
#[path = "support/fixture.rs"]
mod fixture;
#[path = "support/meta_command.rs"]
mod meta_command;
#[path = "support/seeded_log.rs"]
mod seeded_log;
use fixture::{Fixture, Moment};
use meta_command::MetaCommand;
use seeded_log::{SeededLog, TASK};

struct Seed;

impl Seed {
    fn planted(origin: &Path, clone: &Path) {
        Fixture::opened(origin).committed_at(SEED, &SEED_TIME);
        SeededLog::through(OPENED).plant(origin);
        let cloned = Fixture::opened(clone);
        let seed = cloned.committed_at(SEED, &SEED_TIME);
        cloned.branched(TRACKING, &["main"], seed);
        let [head, target] = ORIGIN_HEAD;
        std::fs::write(clone.join(head), target).unwrap();
        let [author, committer] = FOREIGN_IDENTITY;
        cloned.configured("author.email", author);
        cloned.configured("committer.email", committer);
    }

    fn built(target: &Path) -> std::fs::File {
        let [tag, signature] = CARGO_TAG;
        let [artifact, compiled] = ARTIFACT;
        std::fs::create_dir_all(target).unwrap();
        std::fs::write(target.join(tag), signature).unwrap();
        PROFILES.iter().map(|profile| target.join(profile)).for_each(|profile| {
            std::fs::create_dir_all(profile.join(artifact).parent().unwrap()).unwrap();
            std::fs::write(profile.join(LOCK), "").unwrap();
            std::fs::write(profile.join(artifact), compiled).unwrap();
        });
        let building = std::fs::File::open(target.join(PROFILES[1]).join(LOCK)).unwrap();
        building.lock().unwrap();
        building
    }

    fn copied(worktree: &Path) -> bool {
        let [tag, signature] = CARGO_TAG;
        let [artifact, compiled] = ARTIFACT;
        let target = worktree.join(TARGET);
        let read = |path: PathBuf| std::fs::read_to_string(path).unwrap_or_default();
        let [debug, release] = PROFILES.map(|profile| target.join(profile));
        read(target.join(tag)) == signature && read(debug.join(artifact)) == compiled && !release.exists()
    }
}

struct Run(Output);

struct Complaint(String);

struct Written(String);

struct Traced(Run, String);

impl Traced {
    fn run(project: &Project, words: &[&str]) -> Self {
        let trace = tempfile::NamedTempFile::new().unwrap();
        let mut command = MetaCommand::tagged(words);
        command.current_dir(project.clone.path()).env_remove("META_ACTOR").env("USER", LOCAL_NAME);
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
        Fixture::bare(origin.path(), "main");
        Fixture::working(clone.path(), origin.path(), "main");
        std::fs::write(clone.path().join("meta.toml"), "remote = \"origin\"\n").unwrap();
        Self { origin, clone }
    }

    fn seeded(worktrees: &Path) -> Self {
        let project = Self::new();
        Seed::planted(project.origin.path(), project.clone.path());
        std::fs::create_dir_all(project.clone.path().join(".git/info")).unwrap();
        std::fs::write(project.clone.path().join(EXCLUDE), LOCAL_SETTINGS_RULE).unwrap();
        let root = worktrees.display();
        let settings = format!("worktree_root = \"{root}\"\n[identity]\nname = \"{AGENT}\"\nemail = \"{NOREPLY}\"\n");
        std::fs::write(project.clone.path().join("meta.local.toml"), settings).unwrap();
        project
    }

    fn meta(&self, actor: &str, words: &[&str]) -> Run {
        Run(MetaCommand::tagged(words).current_dir(self.clone.path()).env("META_ACTOR", actor).output().unwrap())
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
    assert_eq!(Fixture::opened(project.origin.path()).history_len(LOG), WRITES_LEN);
    assert_eq!(project.meta(WRITER, &ISSUE).actor().as_deref(), Some(WRITER));
    Fixture::opened(project.clone.path()).pointed(Path::new(NOWHERE));
    assert!(project.meta(OWNER, &ISSUE).complaint().0.contains(OFFLINE));
}

#[test]
fn a_claim_failing_after_its_worktree_leaves_no_claim_branch_or_path_so_a_retry_claims_a_listed_seeded_worktree() {
    let worktrees = tempfile::tempdir().unwrap();
    let project = Project::seeded(worktrees.path());
    let _building = Seed::built(&project.clone.path().join(TARGET));
    let claim = || project.meta(OWNER, &[CLAIM, TASK, WORKTREE]);
    std::fs::write(project.clone.path().join(RECORD_DIR), "").unwrap();
    let failed = claim().complaint().0;
    std::fs::remove_file(project.clone.path().join(RECORD_DIR)).unwrap();
    assert!(failed.contains(RECORD_DIR) && Fixture::opened(project.origin.path()).history_len(LOG) == SEEDED_LEN);
    let claimed = claim().to_string();
    let created = Path::new(claimed.lines().last().unwrap_or_default());
    let worktree = Fixture::opened(created);
    let shown = SHOWN_BY_GIT.map(|key| worktree.setting(key));
    let expected = [NOREPLY, NOREPLY, AGENT].map(|value| Some(value.to_owned()));
    assert!(claimed.contains(CLAIMED) && shown == expected && worktree.branch_name() == TASK && Seed::copied(created));
    let stray = worktrees.path().join(AGENT);
    Fixture::opened(project.clone.path()).worktrees(worktrees.path(), &[], AGENT);
    let listed = project.meta(OWNER, &WORKTREES).to_string();
    let named = format!("{UNRECORDED} {}\n", std::fs::canonicalize(stray).unwrap().display());
    assert!(listed.contains(&format!("{TASK} assigned {TASK} ")) && listed.ends_with(&named));
}
