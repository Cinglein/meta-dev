use meta_pull_query::{LiveQuery, LiveReply};
use meta_status::{Github, GithubFailure};

const OFFLINE: &str = "offline";
const MAIN: &str = r#""main""#;
const REQUIRED: &str = r#"["ci"]"#;
const ORPHAN_REPLY: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[
{"number":16,"state":"OPEN","baseRefName":"main","statusCheckRollup":null}]}},
"trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}},{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}}}"#;
const NOTHING_OPEN: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}},
"trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}},{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}}}"#;

#[derive(Clone, Copy, Debug)]
pub enum Scene {
    Offline,
    Orphaned,
    Quiet,
}

impl Scene {
    pub fn answering(self) -> impl FnOnce(&LiveQuery) -> Github {
        move |_| match self {
            Self::Offline => Github::Unreached(GithubFailure::new(OFFLINE)),
            Self::Orphaned => Self::reached(ORPHAN_REPLY),
            Self::Quiet => Self::reached(NOTHING_OPEN),
        }
    }

    fn reached(reply: &str) -> Github {
        let rules = [(serde_json::from_str(MAIN).unwrap(), serde_json::from_str(REQUIRED).unwrap())];
        let pulls = serde_json::from_str::<LiveReply>(reply).unwrap().answered().unwrap();
        Github::Reached(pulls.ruled(rules.into_iter().collect()).unwrap())
    }
}
