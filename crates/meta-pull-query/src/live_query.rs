use std::fmt;

use meta_pulls::Cursor;
use meta_schema::PullRequest;

use crate::pull_alias::PullAlias;

#[derive(Clone, Debug)]
pub struct LiveQuery {
    pull_requests: Vec<PullRequest>,
    cursor: Option<Cursor>,
}

impl LiveQuery {
    const OPENING: &'static str = "query($owner:String!,$name:String!,$after:String){";
    const REPOSITORY: &'static str = "repository(owner:$owner,name:$name){";
    const PULL: &'static str = "number state baseRefName statusCheckRollup{contexts(first:100){pageInfo{hasNextPage} nodes{...on CheckRun{name result:conclusion}...on StatusContext{name:context result:state}}}}";
    const OPEN: &'static str =
        "pullRequests(states:OPEN,first:100,after:$after){pageInfo{hasNextPage endCursor} nodes{";
    const TRUNK: &'static str = "trunk:repository(owner:$owner,name:$name){defaultBranchRef{target{...on Commit{history(first:20){pageInfo{hasNextPage} nodes{checkSuites(first:50){pageInfo{hasNextPage} nodes{conclusion workflowRun{workflow{name}}}}}}}}}}";
    const LINKED: &'static str = "linked:";
    const CLOSING: &'static str = "}";

    #[must_use]
    pub fn new(mut pull_requests: Vec<PullRequest>) -> Self {
        pull_requests.sort_unstable();
        pull_requests.dedup();
        Self { pull_requests, cursor: None }
    }

    #[must_use]
    pub fn after(cursor: Cursor) -> Self {
        Self { pull_requests: Vec::new(), cursor: Some(cursor) }
    }

    #[must_use]
    pub fn field(&self) -> String {
        format!("query={self}")
    }

    #[must_use]
    pub fn cursor_field(&self) -> Option<String> {
        self.cursor.as_ref().map(ToString::to_string)
    }
}

impl fmt::Display for LiveQuery {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}open:{}{}{}}}}}}}", Self::OPENING, Self::REPOSITORY, Self::OPEN, Self::PULL)?;
        self.cursor.as_ref().map_or_else(|| formatter.write_str(Self::TRUNK), |_| Ok(()))?;
        self.pull_requests.first().map_or(Ok(()), |_| write!(formatter, "{}{}", Self::LINKED, Self::REPOSITORY))?;
        self.pull_requests.iter().try_for_each(|pull_request| {
            let number = pull_request.number();
            write!(formatter, "{}:pullRequest(number:{number}){{{}}}", PullAlias::new(*pull_request), Self::PULL)
        })?;
        self.pull_requests.first().map_or(Ok(()), |_| formatter.write_str(Self::CLOSING))?;
        formatter.write_str(Self::CLOSING)
    }
}
