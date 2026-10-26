use std::fmt;

use meta_schema::PullRequest;

use crate::pull_alias::PullAlias;

#[derive(Clone, Debug)]
pub struct LiveQuery(Vec<PullRequest>);

impl LiveQuery {
    const OPENING: &'static str = "query($owner:String!,$name:String!){";
    const REPOSITORY: &'static str = "repository(owner:$owner,name:$name){";
    const OPEN: &'static str = "pullRequests(states:OPEN,first:100){nodes{number}}}";
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

    fn pull(formatter: &mut fmt::Formatter<'_>, pull_request: PullRequest) -> fmt::Result {
        let number = pull_request.number();
        let required = format!("required:isRequired(pullRequestNumber:{number})");
        write!(
            formatter,
            "{}:pullRequest(number:{number}){{state statusCheckRollup{{contexts(first:100){{nodes{{...on CheckRun{{result:conclusion {required}}}...on StatusContext{{result:state {required}}}}}}}}}}}",
            PullAlias::new(pull_request)
        )
    }
}

impl fmt::Display for LiveQuery {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}open:{}{}", Self::OPENING, Self::REPOSITORY, Self::OPEN)?;
        self.0.first().map_or(Ok(()), |_| write!(formatter, "{}{}", Self::LINKED, Self::REPOSITORY))?;
        self.0.iter().try_for_each(|pull_request| Self::pull(formatter, *pull_request))?;
        self.0.first().map_or(Ok(()), |_| formatter.write_str(Self::CLOSING))?;
        formatter.write_str(Self::CLOSING)
    }
}
