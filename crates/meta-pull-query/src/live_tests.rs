use meta_schema::PullRequest;

use meta_pulls::LivePull;

use crate::{LiveData, LiveQuery, LiveReply};

const REPLY: &str = r#"{"data":{"open":{"pullRequests":{"nodes":[{"number":13},{"number":14}]}},"linked":{
"pr13":{"state":"OPEN","statusCheckRollup":{"contexts":{"nodes":[
  {"result":"FAILURE","required":true},{"result":"SUCCESS","required":false}]}}},
"pr14":{"state":"OPEN","statusCheckRollup":{"contexts":{"nodes":[
  {"result":null,"required":true},{"result":"SUCCESS","required":true},{"result":"ERROR","required":false}]}}},
"pr15":{"state":"MERGED","statusCheckRollup":{"contexts":{"nodes":[
  {"result":"SKIPPED","required":false},{"result":"EXPECTED","required":false}]}}},
"pr16":{"state":"CLOSED","statusCheckRollup":null},
"pr17":{"state":"OPEN","statusCheckRollup":{"contexts":{"nodes":[
  {"result":"NEUTRAL","required":true},{"result":"FAILURE","required":false}]}}},
"pr18":{"state":"CLOSED","statusCheckRollup":{"contexts":{"nodes":[{"result":"TIMED_OUT","required":true}]}}},
"pr99":null}},
"errors":[{"type":"NOT_FOUND","path":["linked","pr99"]}]}"#;
const SHOWN: [(&str, &str); 7] = [
    ("18", "closed required red"),
    ("13", "open required red"),
    ("14", "open required pending"),
    ("15", "merged checks pending (none required)"),
    ("16", "closed no checks"),
    ("17", "open required green"),
    ("99", "unknown"),
];
const OPEN_ONLY: &str = "query($owner:String!,$name:String!){open:repository(owner:$owner,name:$name){pullRequests(states:OPEN,first:100){nodes{number}}}}";
const LINKED: &str = "}linked:repository(owner:$owner,name:$name){pr7:pullRequest(number:7){state statusCheckRollup{contexts(first:100){nodes{...on CheckRun{result:conclusion required:isRequired(pullRequestNumber:7)}...on StatusContext{result:state required:isRequired(pullRequestNumber:7)}}}}}pr12:";

struct Parsed;

impl Parsed {
    fn reply() -> LiveData {
        serde_json::from_str::<LiveReply>(REPLY).unwrap().answered().unwrap()
    }

    fn number(text: &str) -> PullRequest {
        text.parse().unwrap()
    }
}

#[test]
fn a_reply_with_a_missing_pull_request_still_reads_and_checks_roll_up_required_first() {
    let data = Parsed::reply();
    let shown = |number| data.pull(Parsed::number(number)).map_or_else(|| "unknown".to_owned(), ToString::to_string);
    assert!(SHOWN.iter().all(|(number, expected)| shown(number) == *expected), "{:?}", SHOWN.map(|(n, _)| shown(n)));
    assert_eq!(data.open().orphans(&[Parsed::number("13")]), [Parsed::number("14")]);
    let red = |number| data.pull(Parsed::number(number)).and_then(LivePull::red).is_some();
    assert!(red("13") && !red("14") && !red("17") && !red("18"));
}

#[test]
fn the_query_asks_for_open_pull_requests_and_each_linked_one_once() {
    assert_eq!(LiveQuery::new(vec![]).field(), format!("query={OPEN_ONLY}"));
    let linked = LiveQuery::new(["12", "7", "12"].map(Parsed::number).to_vec()).to_string();
    assert!(linked.contains(LINKED) && linked.matches("pr12:").count() == 1 && linked.ends_with("12)}}}}}}}"));
}
