use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::time::{Duration, Instant};

use meta_pull_query::LiveQuery;

use super::LiveGithub;
use crate::request::project::Config;

const FAKE_GH: &str = r#"#!/bin/sh
printf '%s\n' "$@" >> "$0.args"
case "$*" in *after=c1*) echo '{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false,"endCursor":"c2"},"nodes":[{"number":4,"state":"OPEN","baseRefName":"main","statusCheckRollup":null}]}}}}'; exit 0 ;; esac
case "$1 $2 $FAKE" in
"api graphql partial"|"api graphql norules") echo '{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":true,"endCursor":"c1"},"nodes":[{"number":3,"state":"OPEN","baseRefName":"main","statusCheckRollup":{"contexts":{"pageInfo":{"hasNextPage":false},"nodes":[{"name":"ci","result":"SUCCESS"}]}}}]}},"trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}},{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}},"errors":[]}'; echo 'gh: not found' >&2; exit 1 ;;
"api graphql garbage") echo 'not json' ;;
"api graphql down") echo 'HTTP 502: Bad Gateway' >&2; exit 1 ;;
"api repos/{owner}/{repo}/rules/branches/main partial") echo '["ci"]' ;;
"api repos/{owner}/{repo}/branches/main partial") echo '["lint"]' ;;
"api repos/{owner}/{repo}/rules/branches/main norules") echo 'HTTP 404: Not Found' >&2; exit 1 ;;
"pr list partial") printf '7\n8\n' ;;
esac
"#;
const ASKED: &str = "api\ngraphql\n-F\nowner={owner}\n-F\nname={repo}\n-f\nquery=query(";
const RULES: &str = "api\nrepos/{owner}/{repo}/rules/branches/main\n--jq\n[.[] | select(.type == \"required_status_checks\") | .parameters.required_status_checks[].context]\n";
const CLASSIC: &str =
    "api\nrepos/{owner}/{repo}/branches/main\n--jq\n[.protection.required_status_checks.checks[]?.context]\n";
const NEXT_PAGE: &str = "-f\nafter=c1\n";
const HEADED: &str = "pr\nlist\n--head\nfeature\n--json\nnumber,isCrossRepository\n--jq\n.[] | select(.isCrossRepository | not) | .number\n";
const SHOWN: &str = "flags:\norphan #3: open, no task\norphan #4: open, no task\n";
const PROTECTED: &str = "open required pending";
const REASONS: [(&str, &str); 3] = [
    ("down", "unknown github: HTTP 502: Bad Gateway"),
    ("garbage", "unknown github: reply not JSON: not json"),
    ("norules", "unknown github: HTTP 404: Not Found"),
];
const WORKFLOWS: [&str; 2] = ["ci", "mutants"];
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
        let workflows = WORKFLOWS.map(|name| name.parse().unwrap()).to_vec();
        let github = LiveGithub::new(program, dir.path().into(), Instant::now() + DEADLINE, workflows);
        Self { dir, github }
    }

    fn asked(&self, scenario: &str) -> String {
        std::fs::read_to_string(self.dir.path().join(format!("{scenario}.args"))).unwrap_or_default()
    }

    fn reason(scenario: &str) -> String {
        let read = Self::new(scenario).github.read(&LiveQuery::new(vec![]));
        read.flag().map(|flag| flag.to_string()).unwrap_or_default()
    }
}

#[test]
fn one_graphql_call_then_the_base_rules_read_a_partial_reply_and_a_branch_lists_its_own_pull_requests() {
    let fake = Fake::new("partial");
    let read = fake.github.read(&LiveQuery::new(vec![]));
    let pulls = fake.github.pulls_from(&"feature".parse().unwrap());
    let state = meta_state::State::default();
    let shown = meta_status::Status::new(&state, &read, chrono::Utc::now().into()).to_string();
    assert!(shown == SHOWN && fake.asked("partial").starts_with(ASKED));
    let asked = fake.asked("partial");
    assert!(asked.ends_with(&format!("\n{RULES}{CLASSIC}{HEADED}")) && asked.contains(NEXT_PAGE));
    assert_eq!(read.sight("3".parse().unwrap()).to_string(), PROTECTED);
    assert_eq!(pulls, ["7", "8"].map(|number| number.parse().unwrap()));
}

#[test]
fn a_project_requires_the_workflows_its_meta_toml_names_and_none_by_default() {
    let required = |settings: &str| {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join("meta.toml"), settings).unwrap();
        let project = crate::request::project::Project::try_from(dir.path()).unwrap();
        LiveGithub::for_project(&project).workflows.iter().map(ToString::to_string).collect::<Vec<_>>()
    };
    assert_eq!(required("remote = \"origin\"\nrequired_workflows = [\"ci\", \"docs\"]"), ["ci", "docs"]);
    assert_eq!(required("remote = \"origin\""), Vec::<String>::new());
}

#[test]
fn an_unreachable_or_unreadable_github_reads_as_unknown_with_its_reason() {
    assert!(REASONS.iter().all(|(scenario, reason)| Fake::reason(scenario) == *reason));
    let fake = Fake::new("down");
    let missing =
        LiveGithub::new(fake.github.program.clone(), Path::new("/nonexistent").into(), Instant::now(), vec![]);
    let unread = missing.read(&LiveQuery::new(vec![]));
    assert!(unread.flag().is_some() && fake.github.pulls_from(&"x".parse().unwrap()).is_empty());
}
