use std::str::FromStr;

use super::Identity;

#[derive(Debug, PartialEq, Eq)]
pub struct Idents(Vec<Identity>);

impl Idents {
    const NAMES: [&'static str; 2] = ["GIT_AUTHOR_IDENT=", "GIT_COMMITTER_IDENT="];

    pub fn foreign(self) -> Option<Identity> {
        self.0.into_iter().find(|identity| !identity.0.noreply())
    }
}

impl FromStr for Idents {
    type Err = std::convert::Infallible;

    fn from_str(listed: &str) -> Result<Self, Self::Err> {
        let shown = |name: &str| listed.lines().rev().find_map(|line| line.strip_prefix(name)).unwrap_or_default();
        let Ok(identities) = Self::NAMES.iter().map(|name| shown(name).parse()).collect();
        Ok(Self(identities))
    }
}
