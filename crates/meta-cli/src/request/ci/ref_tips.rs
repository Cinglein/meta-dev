use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use super::commit::Commit;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RefTips(BTreeMap<String, Commit>);

impl RefTips {
    const REMOTE: &'static str = "refs/remotes/";
    const LOCAL: &'static str = "refs/heads/";

    pub fn pushed_since(&self, before: &Self) -> Self {
        let local: Vec<&Commit> =
            self.0.iter().filter(|(name, _)| name.starts_with(Self::LOCAL)).map(|(_, commit)| commit).collect();
        Self(
            self.0
                .iter()
                .filter(|(name, commit)| {
                    name.starts_with(Self::REMOTE) && before.0.get(*name) != Some(commit) && local.contains(commit)
                })
                .map(|(name, commit)| (name.clone(), commit.clone()))
                .collect(),
        )
    }

    pub fn commits(&self) -> Vec<Commit> {
        let all: Vec<&Commit> = self.0.values().collect();
        all.iter()
            .enumerate()
            .filter(|(index, commit)| all.iter().position(|other| other == *commit) == Some(*index))
            .map(|(_, commit)| (*commit).clone())
            .collect()
    }
}

impl FromStr for RefTips {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            text.lines()
                .filter_map(|line| line.split_once(' '))
                .filter_map(|(name, sha)| sha.parse().ok().map(|commit| (name.to_owned(), commit)))
                .collect(),
        ))
    }
}

impl fmt::Display for RefTips {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|(name, commit)| writeln!(formatter, "{name} {commit}"))
    }
}
