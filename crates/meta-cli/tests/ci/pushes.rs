use std::os::unix::fs::PermissionsExt as _;
use std::process::{Child, Command, Stdio};

use super::{BLOCK, COMMIT, FAILURE, Git, META, NEXT_COMMIT, PASSED_EVENT, POST, PRE, Repo, SESSION, STOP, SUCCESS};

const STACK_PUSH: &str = "gh stack push";
const ELSEWHERE: [&str; 4] = ["push", "-q", "origin", "HEAD:elsewhere"];
const DETACH: [&str; 3] = ["checkout", "-q", "--detach"];
const NOT_PUSHED: &str = "is not pushed";
const UNWRITABLE: &str = "CI state unwritable";
const SESSION_DIR: &str = ".git/meta-ci/ABCDEF1234";
const STATES: &str = "states";
const READ_ONLY: u32 = 0o555;
const WRITABLE: u32 = 0o755;
const REFUSE: &str = "#!/bin/sh\nexit 1\n";
const PRE_RECEIVE: &str = "hooks/pre-receive";
const CI_WATCH: &str = "ci-watch";
const RUNNING: &str = "running";
const PASSED: &str = "passed";

impl Repo {
    fn head(&self) -> String {
        String::from_utf8(Git::run(self.clone.path(), &["rev-parse", "HEAD"]).stdout).unwrap().trim().to_owned()
    }

    fn session_mode(&self, mode: u32) {
        let dir = self.clone.path().join(SESSION_DIR);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(mode)).unwrap();
    }

    fn refuse_pushes(&self) {
        let hook = self._origin.path().join(PRE_RECEIVE);
        std::fs::write(&hook, REFUSE).unwrap();
        std::fs::set_permissions(hook, std::fs::Permissions::from_mode(WRITABLE)).unwrap();
    }

    fn plant_running(&self) {
        let states = self.clone.path().join(SESSION_DIR).join(STATES);
        std::fs::create_dir_all(&states).unwrap();
        std::fs::write(states.join(self.head()), format!("{} {RUNNING}", self.head())).unwrap();
    }

    fn watcher(&self) -> Child {
        let path = format!("{}:{}", self.clone.path().join("bin").display(), std::env::var("PATH").unwrap_or_default());
        Command::new(META)
            .args([CI_WATCH, SESSION, &self.head()])
            .current_dir(self.clone.path())
            .env("PATH", path)
            .env("FAKE_GH_DIR", self.clone.path())
            .env("FAKE_GH_SCENARIO", self.scenario)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
    }
}

#[test]
fn pushes_to_other_names_rejected_pushes_and_unwritable_state_block_stop() {
    let [moved, rejected, unwritable] = [FAILURE, SUCCESS, SUCCESS].map(Repo::new);
    moved.hook(PRE, STACK_PUSH);
    Git::run(moved.clone.path(), &ELSEWHERE);
    moved.hook(POST, STACK_PUSH);
    Git::run(moved.clone.path(), &DETACH);
    rejected.hook(POST, COMMIT);
    rejected.refuse_pushes();
    Git::run(rejected.clone.path(), &NEXT_COMMIT);
    rejected.hook(POST, COMMIT);
    unwritable.session_mode(READ_ONLY);
    unwritable.hook(POST, COMMIT);
    let stops = [&moved, &rejected, &unwritable].map(|repo| repo.hook(STOP, COMMIT));
    unwritable.session_mode(WRITABLE);
    let _: [String; 2] = [&moved, &rejected].map(Repo::settled);
    assert!(stops[0].contains(BLOCK) && stops[1].contains(NOT_PUSHED) && stops[2].contains(UNWRITABLE));
}

#[test]
fn a_dead_watcher_restarts_and_one_commit_gets_one_watcher() {
    let [dead, twinned] = [SUCCESS, SUCCESS].map(Repo::new);
    dead.plant_running();
    dead.hook(POST, COMMIT);
    let twins = [twinned.watcher(), twinned.watcher()].map(|child| child.wait_with_output().unwrap());
    let passes = twins.iter().filter(|twin| String::from_utf8_lossy(&twin.stdout).contains(PASSED)).count();
    assert!(dead.settled().contains(PASSED_EVENT) && passes == 1);
}
