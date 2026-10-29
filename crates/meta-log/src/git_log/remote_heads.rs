use std::str::FromStr;

use super::remote::Remote;
use super::sha::{NotCommitId, Sha};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RemoteHeads(String);

impl RemoteHeads {
    const HEADS: &'static str = "refs/heads/";
    const SEPARATOR: char = '\t';

    pub fn log_tip(&self) -> Result<Option<Sha>, NotCommitId> {
        let named = |name: &str| name.strip_prefix(Self::HEADS) == Some(Remote::BRANCH);
        let tip = self.0.lines().filter_map(|line| line.split_once(Self::SEPARATOR)).find(|(_, name)| named(name));
        tip.map(|(sha, _)| sha.parse()).transpose()
    }

    #[must_use]
    pub fn tips<Tip: FromStr>(&self) -> Vec<Tip> {
        self.0.lines().filter_map(|line| line.parse().ok()).collect()
    }
}

impl FromStr for RemoteHeads {
    type Err = std::convert::Infallible;

    fn from_str(listed: &str) -> Result<Self, Self::Err> {
        Ok(Self(listed.to_owned()))
    }
}
