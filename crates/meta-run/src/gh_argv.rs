use crate::env_policy::EnvPolicy;
use crate::tool::Tool;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GhArgv {
    CommitChecks,
    FailedLog,
}

impl Tool for GhArgv {
    const PROGRAM: &'static str = "gh";
    const POLICY: EnvPolicy = EnvPolicy::Inherit;

    fn words(self) -> &'static [&'static str] {
        match self {
            Self::CommitChecks => &[
                "api",
                "graphql",
                "--jq",
                ".data.repository.object",
                "-f",
                "query=query($owner:String!,$name:String!,$sha:GitObjectID!){repository(owner:$owner,name:$name){object(oid:$sha){...on Commit{checkSuites(first:20,filterBy:{appId:15368}){nodes{status conclusion workflowRun{databaseId} checkRuns(first:100,filterBy:{conclusions:[FAILURE,CANCELLED,TIMED_OUT,STARTUP_FAILURE,ACTION_REQUIRED]}){nodes{databaseId name conclusion steps(first:100){nodes{name conclusion}}}} otherJobs:checkRuns(first:1,filterBy:{conclusions:[SUCCESS,NEUTRAL,SKIPPED,STALE]}){nodes{databaseId name}}}} history(first:8){nodes{checkSuites(first:20,filterBy:{appId:15368}){nodes{status conclusion workflowRun{databaseId} createdAt updatedAt}}}}}}}}",
            ],
            Self::FailedLog => &["run", "view", "--log-failed", "--job"],
        }
    }
}
