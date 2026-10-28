use std::fmt;
use std::process::Child;
use std::time::Instant;

use crate::env_policy::EnvPolicy;
use crate::ran::Ran;
use crate::run::Run;
use crate::run_failure::RunFailure;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupId(u32);

impl GroupId {
    const KILL: &'static str = "kill";
    const END_OF_OPTIONS: &'static str = "--";
    const SEARCH_PATH: [&'static str; 1] = ["PATH"];

    pub fn signalled<'group>(
        groups: impl IntoIterator<Item = &'group Self>,
        signal: &str,
        deadline: Instant,
    ) -> Result<Ran, RunFailure> {
        let kill = Run::new(Self::KILL, EnvPolicy::Only(&Self::SEARCH_PATH)).args([signal, Self::END_OF_OPTIONS]);
        let within = deadline.saturating_duration_since(Instant::now());
        groups.into_iter().fold(kill, |kill, group| kill.arg(group.to_string())).helped(within)
    }
}

impl From<&Child> for GroupId {
    fn from(leader: &Child) -> Self {
        Self(leader.id())
    }
}

#[cfg(test)]
impl std::str::FromStr for GroupId {
    type Err = std::num::ParseIntError;

    fn from_str(leader: &str) -> Result<Self, Self::Err> {
        leader.trim().parse().map(Self)
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "-{}", self.0)
    }
}
