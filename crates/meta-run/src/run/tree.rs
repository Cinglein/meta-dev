mod pid;

use std::collections::BTreeMap;
use std::ops::ControlFlow;
use std::process::Child;
use std::str::FromStr;
use std::time::{Duration, Instant};

use super::Run;
use crate::env_policy::EnvPolicy;
use pid::Pid;

#[derive(Debug, Default)]
pub struct Tree(BTreeMap<Pid, Pid>);

impl Tree {
    const PS: &'static str = "ps";
    const LISTING: [&'static str; 3] = ["-A", "-o", "pid=,ppid=,stat="];
    const KILL: &'static str = "kill";
    const SIGNALS: [&'static str; 2] = ["-TERM", "-KILL"];
    const END_OF_OPTIONS: &'static str = "--";
    const ZOMBIE: &'static str = "Z";
    const GRACE: Duration = Duration::from_secs(1);
    const PAUSE: Duration = Duration::from_millis(10);
    const SEARCH_PATH: [&'static str; 1] = ["PATH"];

    pub fn ended(child: &mut Child) {
        let root = Pid::from(&*child);
        let deadline = Instant::now() + Self::GRACE;
        let signals = std::iter::once(Self::SIGNALS[0]).chain(std::iter::repeat(Self::SIGNALS[1]));
        let live = signals.take_while(|_| Instant::now() < deadline);
        let _swept = live.into_iter().try_for_each(|signal| Self::listed(deadline).signalled(root, signal, deadline));
        Run::alone(child);
    }

    fn listed(deadline: Instant) -> Self {
        let ps = Run { ended: Run::alone, ..Run::new(Self::PS, EnvPolicy::Only(&Self::SEARCH_PATH)) };
        let Ok(tree) = ps.args(Self::LISTING).stdout(deadline).unwrap_or_default().parse();
        tree
    }

    fn signalled(&self, root: Pid, signal: &str, deadline: Instant) -> ControlFlow<()> {
        let below = self.below(root);
        below.first().map_or(ControlFlow::Break(()), |_| {
            let kill = Run { ended: Run::alone, ..Run::new(Self::KILL, EnvPolicy::Only(&Self::SEARCH_PATH)) };
            let kill = below.iter().fold(kill.args([signal, Self::END_OF_OPTIONS]), |kill, pid| pid.appended(kill));
            drop(kill.ran(deadline));
            std::thread::sleep(Self::PAUSE);
            ControlFlow::Continue(())
        })
    }

    fn below(&self, root: Pid) -> Vec<Pid> {
        let children = |level: &Vec<Pid>| {
            let next: Vec<Pid> =
                self.0.iter().filter(|(_, parent)| level.contains(parent)).map(|(pid, _)| *pid).collect();
            (!next.is_empty()).then_some(next)
        };
        std::iter::successors(Some(vec![root]), children).skip(1).flatten().collect()
    }
}

impl FromStr for Tree {
    type Err = std::convert::Infallible;

    fn from_str(listed: &str) -> Result<Self, Self::Err> {
        let live = listed.lines().filter_map(|line| match line.split_whitespace().collect::<Vec<_>>()[..] {
            [pid, parent, state] if !state.starts_with(Self::ZOMBIE) => Some((pid.parse().ok()?, parent.parse().ok()?)),
            _ => None,
        });
        Ok(Self(live.collect()))
    }
}
