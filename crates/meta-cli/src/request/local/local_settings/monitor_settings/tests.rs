use std::path::{Path, PathBuf};

use super::MonitorSettings;
use crate::request::project::{Config, HeartbeatSettings};

const TOO_WIDE: [&str; 8] = ["", ".", "/", "~", "..", "~/..", "~/gone/../..", "~/p"];
const TOO_LONG: [&str; 6] = [
    "grace_seconds = 31",
    "probe_seconds = 31",
    "start_slack_seconds = 31",
    "watch_percent = 101",
    "watch_percent = 0",
    "stop = [\"silent\"]",
];
const UNPACED: &str = "mark_seconds = 299";
const SCRATCH: &str = "scratch = [\"~/scratch\", \"work\"]\nspared = []";
const CLAUDE: &str = "claude";
const HOME: &str = "h";
const CHECKOUT: &str = "p";
const WORKTREES: &str = "p-worktrees";
const SCRATCH_DIRS: [&str; 2] = ["scratch", "p/work"];

struct Home<'home>(&'home Path);

impl Home<'_> {
    fn merged(&self, text: &str) -> Option<MonitorSettings> {
        self.placed(text, &self.0.join(WORKTREES))
    }

    fn placed(&self, text: &str, worktrees: &Path) -> Option<MonitorSettings> {
        MonitorSettings::merged(toml::from_str(text).unwrap(), &self.0.join(CHECKOUT), &[worktrees], self.0).ok()
    }
}

#[test]
fn scratch_roots_wider_than_the_project_and_unbounded_waits_are_refused_and_spared_keeps_its_floor() {
    let scratch = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(scratch.path().join(HOME).join(CHECKOUT)).unwrap();
    let home = std::fs::canonicalize(scratch.path().join(HOME)).unwrap();
    let place = Home(&home);
    let wide = TOO_WIDE.map(|root| place.merged(&format!("scratch = [{root:?}]")).is_none());
    let long = TOO_LONG.map(|text| place.merged(text).is_none());
    let heartbeat = HeartbeatSettings::try_from(&Config::default()).unwrap();
    let unpaced = place.merged(UNPACED).and_then(|settings| settings.paced(&heartbeat));
    let long = [long.as_slice(), &[place.placed("", &home).is_none(), unpaced.is_none()]].concat();
    let kept = place.merged(SCRATCH).unwrap();
    let roots: Vec<PathBuf> = kept.roots().iter().skip(2).map(|root| root.path().to_path_buf()).collect();
    assert!(wide.iter().chain(&long).all(|refused| *refused));
    assert_eq!(roots, SCRATCH_DIRS.map(|dir| home.join(dir)));
    assert!(kept.spared().iter().any(|name| name.text() == CLAUDE) && kept.paced(&heartbeat).is_some());
}
