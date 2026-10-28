use std::fmt;

use meta_schema::PullRequest;

use crate::pull_alias::PullAlias;

#[derive(Clone, Debug)]
pub struct LiveQuery(Vec<PullRequest>);

impl LiveQuery {
    const OPENING: &'static str = "query($owner:String!,$name:String!){";
    const REPOSITORY: &'static str = "repository(owner:$owner,name:$name){";
    const PULL: &'static str = "number state baseRefName headRefName statusCheckRollup{contexts(first:100){pageInfo{hasNextPage} nodes{...on CheckRun{name result:conclusion}...on StatusContext{name:context result:state}}}}";
    const OPEN: &'static str = "pullRequests(states:OPEN,first:100){pageInfo{hasNextPage} nodes{";
    const TRUNK: &'static str = "trunk:repository(owner:$owner,name:$name){defaultBranchRef{target{...on Commit{history(first:20){pageInfo{hasNextPage} nodes{checkSuites(first:50){pageInfo{hasNextPage} nodes{conclusion workflowRun{workflow{name}}}}}}}}}}";
    const LINKED: &'static str = "linked:";
    const CLOSING: &'static str = "}";

    #[must_use]
    pub fn new(mut pull_requests: Vec<PullRequest>) -> Self {
        pull_requests.sort_unstable();
        pull_requests.dedup();
        Self(pull_requests)
    }

    #[must_use]
    pub fn field(&self) -> String {
        format!("query={self}")
    }
}

impl fmt::Display for LiveQuery {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}open:{}{}{}}}}}}}", Self::OPENING, Self::REPOSITORY, Self::OPEN, Self::PULL)?;
        formatter.write_str(Self::TRUNK)?;
        self.0.first().map_or(Ok(()), |_| write!(formatter, "{}{}", Self::LINKED, Self::REPOSITORY))?;
        self.0.iter().try_for_each(|pull_request| {
            let number = pull_request.number();
            write!(formatter, "{}:pullRequest(number:{number}){{{}}}", PullAlias::new(*pull_request), Self::PULL)
        })?;
        self.0.first().map_or(Ok(()), |_| formatter.write_str(Self::CLOSING))?;
        formatter.write_str(Self::CLOSING)
    }
}
