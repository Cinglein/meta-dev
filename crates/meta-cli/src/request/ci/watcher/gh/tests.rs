use std::os::unix::fs::PermissionsExt as _;
use std::time::{Duration, Instant};

use tempfile::TempDir;

use super::Gh;
use crate::request::ci::Commit;
use crate::request::project::{CiSettings, Config};
use crate::request::repository::Repository;

const REPLY: &str = r#"{"checkSuites":{"nodes":[{"status":"COMPLETED","conclusion":"FAILURE","workflowRun":{"databaseId":1},"checkRuns":{"nodes":[{"databaseId":11,"name":"ci","conclusion":"FAILURE"}]}}]}}"#;
const SCRIPT: &str = r#"#!/bin/sh
case "$1" in
api) cat "$(dirname "$0")/reply" ;;
run) [ -e "$(dirname "$0")/ok" ] || { echo boom >&2; exit 1; }
  sleep 0.05
  printf 'ci\tstep\ttest slow ... FAILED\n' ;;
esac
"#;
const URL: &str = "https://github.com/o/r.git";
const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const WITHIN: Duration = Duration::from_secs(30);
const LOG_LINE: &str = "test slow ... FAILED";
const DOWNLOAD_FAILED: &str = "gh failed: boom";

struct Fixture {
    dir: TempDir,
    settings: CiSettings,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let program = dir.path().join("gh");
        std::fs::write(&program, SCRIPT).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(dir.path().join("reply"), REPLY).unwrap();
        let text = format!("[ci]\nprogram = \"{}\"\ngh_millis = 10\n", program.display());
        Self { dir, settings: toml::from_str::<Config>(&text).unwrap().ci_settings() }
    }

    fn allow_downloads(&self) {
        std::fs::write(self.dir.path().join("ok"), "").unwrap();
    }

    fn excerpt(&self) -> String {
        let gh = Gh::new(&self.settings, self.dir.path().to_path_buf(), Instant::now() + WITHIN);
        let repository: Repository = URL.parse().unwrap();
        let Ok(commit): Result<Commit, _> = COMMIT.parse();
        gh.failed_logs(&repository, &[commit]).excerpt(self.settings.log_window()).to_string()
    }
}

#[test]
fn a_log_download_may_outlast_the_call_limit_and_a_failed_one_says_so() {
    let fixture = Fixture::new();
    let failed = fixture.excerpt();
    fixture.allow_downloads();
    let slow = fixture.excerpt();
    assert!(slow.contains(LOG_LINE) && failed.contains(DOWNLOAD_FAILED), "{failed}\n{slow}");
}
