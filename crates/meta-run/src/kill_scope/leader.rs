use std::fmt;
use std::process::Child;
use std::time::{Duration, Instant};

use crate::env_policy::EnvPolicy;
use crate::run::Run;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Leader(u32);

impl Leader {
    const KILL: &'static str = "kill";
    const SIGNAL: &'static str = "-KILL";
    const END_OF_OPTIONS: &'static str = "--";
    const GRACE: Duration = Duration::from_secs(1);
    const SEARCH_PATH: [&'static str; 1] = ["PATH"];

    pub fn ended(self) {
        let kill = Run::new(Self::KILL, EnvPolicy::Only(&Self::SEARCH_PATH)).args([Self::SIGNAL, Self::END_OF_OPTIONS]);
        drop(kill.arg(self.to_string()).ran(Instant::now() + Self::GRACE));
    }
}

impl From<&Child> for Leader {
    fn from(child: &Child) -> Self {
        Self(child.id())
    }
}

impl fmt::Display for Leader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "-{}", self.0)
    }
}
