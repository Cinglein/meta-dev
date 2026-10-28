use std::time::{Duration, Instant};

use meta_run::{EnvPolicy::Inherit, Run};

pub struct Pids;

impl Pids {
    const KILL: &str = "kill";
    const ALIVE: &str = "-0";
    const POLL: Duration = Duration::from_millis(10);
    const WAIT: Duration = Duration::from_secs(1);

    pub fn gone(pids: &tempfile::NamedTempFile, recorders: &[&str]) -> bool {
        let pids = std::fs::read_to_string(pids.path()).unwrap_or_default();
        let alive = |pid| Run::new(Self::KILL, Inherit).args([Self::ALIVE, pid]).stdout(Instant::now() + Self::WAIT);
        let polls = (0..Self::WAIT.as_millis() / Self::POLL.as_millis()).map(|_| std::thread::sleep(Self::POLL));
        pids.lines().count() == recorders.len()
            && polls.into_iter().any(|()| pids.lines().all(|pid| alive(pid).is_err()))
    }
}
