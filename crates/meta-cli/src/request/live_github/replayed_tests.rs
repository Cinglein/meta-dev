use std::path::Path;
use std::time::{Duration, Instant};

use meta_pull_query::{LiveQuery, LiveReply};

use super::LiveGithub;
use crate::request::project::{CiSettings, Config};

const UNBUILT: &str = "gh-replay is not built beside this test binary; run the package's tests, not only --lib";
const PROGRAM: &str = "gh-replay";
const FIRST_PAGE: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":true,"endCursor":"c1"},"nodes":[{"number":3,"state":"OPEN","baseRefName":"main","statusCheckRollup":{"contexts":{"pageInfo":{"hasNextPage":false},"nodes":[{"name":"ci","result":"SUCCESS"}]}}}]}},"linked":{"pr40":{"number":40,"state":"MERGED","baseRefName":"gone","statusCheckRollup":null}},"trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}},{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}},"errors":[]}
"#;
const NEXT_PAGE: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false,"endCursor":"c2"},"nodes":[{"number":4,"state":"OPEN","baseRefName":"main","statusCheckRollup":null},{"number":5,"state":"OPEN","baseRefName":"odd","statusCheckRollup":null}]}}}}
"#;
const PARTIAL: &str = "gh: not found\n";
const RULES: &str =
    "[.[] | select(.type == \"required_status_checks\") | .parameters.required_status_checks[].context]";
const CLASSIC: &str = "[.protection.required_status_checks.checks[]?.context]";
const RULESETS: &str = "repos/{owner}/{repo}/rules/branches/";
const PROTECTION: &str = "repos/{owner}/{repo}/branches/";
const HEADED: [&str; 8] = [
    "pr",
    "list",
    "--head",
    "feature",
    "--json",
    "number,isCrossRepository",
    "--jq",
    ".[] | select(.isCrossRepository | not) | .number",
];
const SHOWN: &str = "flags:\norphan #3: open, no task\norphan #4: open, no task\norphan #5: open, no task\n";
const SIGHTS: [&str; 3] = ["open required pending", "merged", "open no checks"];
const REASONS: [(&str, &str); 3] = [
    ("down", "unknown github: HTTP 502: Bad Gateway"),
    ("garbage", "unknown github: reply not JSON: not json"),
    ("norules", "unknown github: HTTP 404: Not Found"),
];
const WORKFLOWS: [&str; 2] = ["ci", "mutants"];
const DEADLINE: Duration = Duration::from_secs(5);
const SUCCEEDED: &str = "0";
const FAILED: &str = "1";
const CLAIMED: &str = "0";

struct Fake {
    dir: tempfile::TempDir,
    github: LiveGithub,
    replies: Vec<[String; 5]>,
}

impl Fake {
    fn new(scenario: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let replies = Self::replies(scenario);
        replies.iter().try_for_each(|reply| Self::recorded(dir.path(), reply)).unwrap();
        let tests = std::env::current_exe().unwrap();
        let replay = tests.parent().and_then(Path::parent).map(|built| built.join(PROGRAM));
        let replay = replay.filter(|built| built.exists()).expect(UNBUILT);
        let config: Config = toml::from_str(&format!("[ci]\nprogram = \"{}\"", replay.display())).unwrap();
        let program = CiSettings::try_from(&config).unwrap().program().clone();
        let workflows = WORKFLOWS.map(|name| name.parse().unwrap()).to_vec();
        let github = LiveGithub::new(program, dir.path().into(), Instant::now() + DEADLINE, workflows)
            .naming(Some("o/r".parse().unwrap()));
        Self { dir, github, replies }
    }

    fn replies(scenario: &str) -> Vec<[String; 5]> {
        let first = LiveQuery::new(vec![]);
        let following = serde_json::from_str::<LiveReply>(FIRST_PAGE).unwrap().answered().unwrap().following().unwrap();
        let paged = |query: &LiveQuery| {
            let words =
                LiveGithub::GRAPHQL.map(String::from).into_iter().chain([LiveGithub::QUERY.into(), query.field()]);
            let after = query.cursor_field().into_iter().flat_map(|cursor| [LiveGithub::QUERY.into(), cursor]);
            Self::argv(words.chain(after))
        };
        let rules = |endpoint: &str, base: &str, filter: &str| {
            Self::argv([LiveGithub::API.into(), format!("{endpoint}{base}"), LiveGithub::FILTER.into(), filter.into()])
        };
        let reply = |name: &str, argv: String, out: &str, err: &str, status: &str| {
            [name.into(), argv, out.into(), err.into(), status.into()]
        };
        let opened = reply("01-first-page", paged(&first), FIRST_PAGE, PARTIAL, FAILED);
        let headed = Self::argv(HEADED.map(String::from));
        match scenario {
            "down" => vec![reply("01-down", paged(&first), "", "HTTP 502: Bad Gateway\n", FAILED)],
            "garbage" => vec![reply("01-garbage", paged(&first), "not json\n", "", SUCCEEDED)],
            "norules" => vec![
                opened,
                reply("02-next-page", paged(&following), NEXT_PAGE, "", SUCCEEDED),
                reply("03-main-rules", rules(RULESETS, "main", RULES), "", "HTTP 404: Not Found\n", FAILED),
                reply("04-main-protection", rules(PROTECTION, "main", CLASSIC), "[\"lint\"]\n", "", SUCCEEDED),
                reply("05-odd-rules", rules(RULESETS, "odd", RULES), "[]\n", "", SUCCEEDED),
                reply("06-odd-protection", rules(PROTECTION, "odd", CLASSIC), "[]\n", "", SUCCEEDED),
            ],
            _ => vec![
                opened,
                reply("02-next-page", paged(&following), NEXT_PAGE, "", SUCCEEDED),
                reply("03-main-rules", rules(RULESETS, "main", RULES), "[\"ci\"]\n", "", SUCCEEDED),
                reply("04-main-protection", rules(PROTECTION, "main", CLASSIC), "[\"lint\"]\n", "", SUCCEEDED),
                reply("05-odd-rules", rules(RULESETS, "odd", RULES), "[]\n", "", SUCCEEDED),
                reply("06-odd-protection", rules(PROTECTION, "odd", CLASSIC), "", "gh: Not Found (HTTP 404)\n", FAILED),
                reply("07-headed", headed, "7\n8\n", "", SUCCEEDED),
            ],
        }
    }

    fn argv(words: impl IntoIterator<Item = String>) -> String {
        serde_json::to_string(&[PROGRAM.to_string()].into_iter().chain(words).collect::<Vec<_>>()).unwrap()
    }

    fn recorded(dir: &Path, [name, argv, out, err, status]: &[String; 5]) -> std::io::Result<()> {
        let stem = dir.join(name);
        [("argv", argv), ("out", out), ("err", err), ("status", status)]
            .iter()
            .try_for_each(|(extension, text)| std::fs::write(stem.with_extension(extension), text))
    }

    fn unclaimed(&self) -> Vec<String> {
        let claimed = |name: &String| self.dir.path().join(format!("{name}.{CLAIMED}")).exists();
        self.replies.iter().map(|[name, ..]| name.clone()).filter(|name| !claimed(name)).collect()
    }

    fn reason(scenario: &str) -> String {
        let read = Self::new(scenario).github.read(&LiveQuery::new(vec![]));
        read.flag().map(|flag| flag.to_string()).unwrap_or_default()
    }
}

#[test]
fn one_graphql_call_per_page_then_the_base_rules_read_a_partial_reply_and_a_branch_lists_its_own_pull_requests() {
    let fake = Fake::new("partial");
    let read = fake.github.read(&LiveQuery::new(vec![]));
    let pulls = fake.github.pulls_from(&"feature".parse().unwrap());
    let state = meta_state::State::default();
    let shown = meta_status::Status::new(&state, &read, chrono::Utc::now().into()).to_string();
    assert_eq!((shown.as_str(), fake.unclaimed()), (SHOWN, Vec::<String>::new()));
    assert_eq!(["3", "40", "5"].map(|number| read.sight(number.parse().unwrap()).to_string()), SIGHTS);
    assert_eq!(pulls, ["7", "8"].map(|number| number.parse().unwrap()));
}

#[test]
fn an_unreachable_or_unreadable_github_reads_as_unknown_with_its_reason() {
    assert_eq!(REASONS.map(|(scenario, _)| Fake::reason(scenario)), REASONS.map(|(_, reason)| reason));
    let fake = Fake::new("down");
    let missing =
        LiveGithub::new(fake.github.program.clone(), Path::new("/nonexistent").into(), Instant::now(), vec![]);
    let unread = missing.read(&LiveQuery::new(vec![]));
    assert!(unread.flag().is_some() && fake.github.pulls_from(&"x".parse().unwrap()).is_empty());
}
