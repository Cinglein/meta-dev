use std::convert::Infallible;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct JobGuard(Option<String>);

impl JobGuard {
    const OPEN: &str = "${{";
    const CLOSE: &str = "}}";
    const NEEDS: &str = "needs.";
    const EMPTY_PLAN: &str = ".outputs.shards != '[]'";

    pub fn planner(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

impl FromStr for JobGuard {
    type Err = Infallible;

    fn from_str(condition: &str) -> Result<JobGuard, Infallible> {
        let bare = condition.trim();
        let bare = bare.strip_prefix(Self::OPEN).and_then(|inner| inner.strip_suffix(Self::CLOSE)).unwrap_or(bare);
        let planner = bare.trim().strip_prefix(Self::NEEDS).and_then(|rest| rest.strip_suffix(Self::EMPTY_PLAN));
        Ok(JobGuard(planner.filter(|job| !job.contains(['.', ' '])).map(str::to_owned)))
    }
}
