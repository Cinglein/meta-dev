use std::path::Path;

use super::MonitorSettings;

const TOO_WIDE: [&str; 7] = ["", ".", "/", "~", "..", "~/..", "~/gone/../.."];
const TOO_LONG: [&str; 3] = ["grace_seconds = 31", "probe_seconds = 31", "watch_percent = 101"];
const SCRATCH: &str = "scratch = [\"~/scratch\"]\nspared = []";
const CLAUDE: &str = "claude";
const HOME: &str = "h";
const CHECKOUT: &str = "p";
const SCRATCH_DIR: &str = "scratch";

struct Home<'home>(&'home Path);

impl Home<'_> {
    fn merged(&self, text: &str) -> Option<MonitorSettings> {
        let checkout = self.0.join(CHECKOUT);
        MonitorSettings::merged(toml::from_str(text).unwrap(), &checkout, &checkout, self.0).ok()
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
    let kept = place.merged(SCRATCH).unwrap();
    assert!(wide.iter().chain(&long).all(|refused| *refused));
    assert_eq!(kept.roots().last().map(|root| root.path().to_path_buf()), Some(home.join(SCRATCH_DIR)));
    assert!(kept.spared().iter().any(|name| name.text() == CLAUDE));
}
