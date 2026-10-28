use std::fmt;

use crate::env_policy::EnvPolicy;
use crate::kill_scope::KillScope;
use crate::leader::Leader;
use crate::run::Run;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KillTarget {
    Process(Leader),
    Group(Leader),
}

impl KillTarget {
    const KILL: &'static str = "kill";
    const SIGNAL: &'static str = "-KILL";
    const END_OF_OPTIONS: &'static str = "--";
    const SEARCH_PATH: [&'static str; 1] = ["PATH"];

    pub fn ended(self) {
        let kill = Run::new(Self::KILL, EnvPolicy::Only(&Self::SEARCH_PATH)).scoped(KillScope::Process);
        drop(kill.args([Self::SIGNAL, Self::END_OF_OPTIONS]).arg(self.to_string()).spawned());
    }
}

impl fmt::Display for KillTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Process(leader) => write!(formatter, "{leader}"),
            Self::Group(leader) => write!(formatter, "-{leader}"),
        }
    }
}
