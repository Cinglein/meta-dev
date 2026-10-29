use meta_pulls::RuledPull;
use meta_schema::PullRequest;

use crate::{LiveData, LiveQuery, LiveReply, Pulls, Rulebook};

const REPLY: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":true},"nodes":[
{"number":13,"state":"OPEN","baseRefName":"main","statusCheckRollup":null},
{"number":21,"state":"OPEN","baseRefName":"main","statusCheckRollup":{"contexts":{"pageInfo":{"hasNextPage":false},"nodes":[{"name":"ci","result":"SUCCESS"}]}}}]}},
"linked":{
"pr13":{"number":13,"state":"OPEN","baseRefName":"main","statusCheckRollup":{"contexts":{"pageInfo":{"hasNextPage":false},"nodes":[
  {"name":"ci","result":"FAILURE"},{"name":"docs","result":"SUCCESS"}]}}},
"pr14":{"number":14,"state":"OPEN","baseRefName":"main","statusCheckRollup":{"contexts":{"pageInfo":{"hasNextPage":false},"nodes":[
  {"name":"docs","result":"SUCCESS"}]}}},
"pr15":{"number":15,"state":"MERGED","baseRefName":"dev","statusCheckRollup":{"contexts":{"pageInfo":{"hasNextPage":true},"nodes":[
  {"name":"x","result":"SKIPPED"},{"name":"y","result":"EXPECTED"}]}}},
"pr16":{"number":16,"state":"CLOSED","baseRefName":"dev","statusCheckRollup":null},
"pr17":{"number":17,"state":"OPEN","baseRefName":"main","statusCheckRollup":{"contexts":{"pageInfo":{"hasNextPage":false},"nodes":[
  {"name":"ci","result":"NEUTRAL"},{"name":"docs","result":"FAILURE"}]}}},
"pr18":{"number":18,"state":"CLOSED","baseRefName":"main","statusCheckRollup":{"contexts":{"pageInfo":{"hasNextPage":false},"nodes":[
  {"name":"ci","result":"TIMED_OUT"}]}}},
"pr19":{"number":19,"state":"OPEN","baseRefName":"main","statusCheckRollup":{"contexts":{"pageInfo":{"hasNextPage":true},"nodes":[
  {"name":"ci","result":"SUCCESS"}]}}},
"pr99":null}},
"errors":[{"type":"NOT_FOUND","path":["linked","pr99"]}]}"#;
const SHOWN: [(&str, &str); 10] = [
    ("13", "open required red"),
    ("14", "open required pending"),
    ("15", "merged"),
    ("16", "closed"),
    ("17", "open required green"),
    ("18", "closed required red"),
    ("19", "open checks truncated"),
    ("21", "open required green"),
    ("22", "unknown"),
    ("99", "unknown"),
];
const MAIN_REQUIRES: &str = r#"["ci"]"#;
const OPEN_ONLY: &str = "query=query($owner:String!,$name:String!,$after:String){open:repository(owner:$owner,name:$name){pullRequests(states:OPEN,first:100,after:$after){pageInfo{hasNextPage endCursor} nodes{number state baseRefName headRefName isCrossRepository statusCheckRollup{contexts(first:100){pageInfo{hasNextPage} nodes{...on CheckRun{name result:conclusion}...on StatusContext{name:context result:state}}}}}}}trunk:repository(owner:$owner,name:$name){defaultBranchRef{target{...on Commit{history(first:20){pageInfo{hasNextPage} nodes{checkSuites(first:50){pageInfo{hasNextPage} nodes{conclusion workflowRun{workflow{name}}}}}}}}}}}";
const LINKED: &str = "}linked:repository(owner:$owner,name:$name){pr7:pullRequest(number:7){number state baseRefName headRefName isCrossRepository statusCheckRollup{";

struct Parsed;

impl Parsed {
    fn pulls() -> Pulls {
        serde_json::from_str::<LiveReply>(REPLY).unwrap().answered().unwrap()
    }

    fn reply() -> LiveData {
        let rules = [("main", MAIN_REQUIRES)].map(|(base, required)| {
            (serde_json::from_str(&format!("{base:?}")).unwrap(), serde_json::from_str(required).unwrap())
        });
        Self::pulls().ruled(rules.into_iter().collect()).unwrap()
    }

    fn number(text: &str) -> PullRequest {
        text.parse().unwrap()
    }
}

#[test]
fn checks_roll_up_over_the_bases_required_names_and_a_missing_one_is_pending() {
    let data = Parsed::reply();
    let shown =
        |number| data.pull(Parsed::number(number)).map_or_else(|| "unknown".to_owned(), |pull| pull.to_string());
    assert!(SHOWN.iter().all(|(number, expected)| shown(number) == *expected), "{:?}", SHOWN.map(|(n, _)| shown(n)));
    let red = |number| data.pull(Parsed::number(number)).and_then(RuledPull::red).is_some();
    assert!(red("13") && !red("14") && !red("17") && !red("18") && !red("19"));
}

#[test]
fn open_pull_requests_read_as_orphans_and_truncation_is_reported() {
    let data = Parsed::reply();
    assert_eq!(data.orphans(&[Parsed::number("13")]), [Parsed::number("21")]);
    let truncated = |number| data.pull(Parsed::number(number)).and_then(RuledPull::truncated);
    assert!(
        data.truncated().is_some()
            && truncated("19").is_some()
            && truncated("15").is_none()
            && data.pull(Parsed::number("13")).and_then(RuledPull::truncated).is_none()
    );
    assert!(Parsed::pulls().ruled(Rulebook::default()).is_none());
}

#[test]
fn the_query_asks_for_open_pull_requests_and_each_linked_one_once() {
    assert_eq!(LiveQuery::new(vec![]).field(), OPEN_ONLY);
    let linked = LiveQuery::new(["12", "7", "12"].map(Parsed::number).to_vec()).field();
    assert!(linked.contains(LINKED) && linked.matches("pr12:").count() == 1 && linked.ends_with("state}}}}}}}"));
}
