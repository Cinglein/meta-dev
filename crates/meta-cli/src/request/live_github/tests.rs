use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::time::{Duration, Instant};

use meta_pull_query::LiveQuery;

use super::LiveGithub;
use crate::request::project::Config;

const FAKE_GH: &str = r#"#!/bin/sh
printf '%s\n' "$@" >> "$0.args"
case "$1 $FAKE" in
"api partial") echo '{"data":{"open":{"pullRequests":{"nodes":[{"number":3}]}}},"errors":[]}'; echo 'gh: not found' >&2; exit 1 ;;
"api down") echo 'HTTP 502: Bad Gateway' >&2; exit 1 ;;
"pr partial") printf '7\n8\n' ;;
esac
"#;
const ASKED: &str = "api\ngraphql\n-F\nowner={owner}\n-F\nname={repo}\n-f\nquery=query(";
const HEADED: &str = "pr\nlist\n--head\nfeature\n--json\nnumber\n--jq\n.[].number\n";
const DEADLINE: Duration = Duration::from_secs(5);

struct Fake {
    dir: tempfile::TempDir,
    github: LiveGithub,
}

impl Fake {
    fn new(scenario: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join(scenario);
        std::fs::write(&script, FAKE_GH.replace("$FAKE", scenario)).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let config: Config = toml::from_str(&format!("[ci]\nprogram = {:?}", script.display().to_string())).unwrap();
        let program = config.ci_settings().program().clone();
        let github = LiveGithub::new(program, dir.path().into(), Instant::now() + DEADLINE);
        Self { dir, github }
    }

    fn asked(&self, scenario: &str) -> String {
        std::fs::read_to_string(self.dir.path().join(format!("{scenario}.args"))).unwrap_or_default()
    }
}

#[test]
fn one_graphql_call_reads_a_partial_reply_and_lists_a_branchs_pull_requests() {
    let fake = Fake::new("partial");
    let read = fake.github.read(&LiveQuery::new(vec![]));
    let pulls = fake.github.pulls_from(&"feature".parse().unwrap());
    assert!(read.orphans(&[]) == ["3".parse().unwrap()] && read.failure().is_none());
    assert!(fake.asked("partial").starts_with(ASKED) && fake.asked("partial").ends_with(HEADED));
    assert_eq!(pulls, ["7", "8"].map(|number| number.parse().unwrap()));
}

#[test]
fn an_unreachable_or_missing_gh_reads_as_unknown_with_its_reason() {
    let fake = Fake::new("down");
    let down = &fake.github;
    let reason = down.read(&LiveQuery::new(vec![])).failure().map(ToString::to_string);
    assert_eq!(reason.as_deref(), Some("HTTP 502: Bad Gateway"));
    let missing = LiveGithub::new(down.program.clone(), Path::new("/nonexistent").into(), Instant::now());
    let unread = missing.read(&LiveQuery::new(vec![]));
    assert!(unread.failure().is_some() && down.pulls_from(&"x".parse().unwrap()).is_empty());
}
