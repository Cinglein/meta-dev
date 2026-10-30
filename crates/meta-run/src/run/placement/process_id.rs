use std::fmt;
use std::process::Child;
use std::time::Duration;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use crate::env_policy::EnvPolicy;
use crate::ran::Ran;
use crate::run::Run;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessId(u32);

impl ProcessId {
    const PGREP: &'static str = "pgrep";
    const PARENT: &'static str = "-P";
    const SEARCH_PATH: [&'static str; 1] = ["PATH"];
    const LOOKUP: Duration = Duration::from_secs(1);

    pub fn ended(child: &mut Child) {
        let descendants = Self::descendants(&[Self(child.id())]);
        Run::alone(child);
        descendants.into_iter().for_each(Self::killed);
    }

    fn killed(self) {
        let _delivered = i32::try_from(self.0).ok().map(|pid| kill(Pid::from_raw(pid), Signal::SIGKILL));
    }

    fn descendants(parents: &[Self]) -> Vec<Self> {
        let listed = parents.iter().map(ToString::to_string).collect::<Vec<_>>().join(",");
        let lookup = Run::new(Self::PGREP, EnvPolicy::Only(&Self::SEARCH_PATH)).args([Self::PARENT, &listed]);
        let found = (!parents.is_empty()).then(|| lookup.helped(Self::LOOKUP).and_then(Ran::succeeded));
        let children: Vec<Self> = found
            .and_then(Result::ok)
            .map(|text| text.split_whitespace().filter_map(|word| word.parse().ok().map(Self)).collect())
            .unwrap_or_default();
        let deeper = children.first().map_or_else(Vec::new, |_| Self::descendants(&children));
        children.into_iter().chain(deeper).collect()
    }
}

impl fmt::Display for ProcessId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
