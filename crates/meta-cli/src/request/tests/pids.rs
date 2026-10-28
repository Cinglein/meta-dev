use std::path::Path;
use std::time::{Duration, Instant};

use meta_run::{EnvPolicy, Run};

pub struct Pids;

impl Pids {
    const ALIVE: [&str; 2] = ["kill", "-0"];
    const POLL: Duration = Duration::from_millis(10);
    const PATIENT: Duration = Duration::from_secs(2);

    pub fn gone(pids: &Path, recorders: &[&str]) -> bool {
        let pids = std::fs::read_to_string(pids).unwrap_or_default();
        let alive = |pid| {
            Run::new(Self::ALIVE[0], EnvPolicy::Inherit)
                .args([Self::ALIVE[1], pid])
                .stdout(Instant::now() + Self::PATIENT)
        };
        let polls = (0..Self::PATIENT.as_millis() / Self::POLL.as_millis()).map(|_| std::thread::sleep(Self::POLL));
        pids.lines().count() == recorders.len()
            && polls.into_iter().any(|()| pids.lines().all(|pid| alive(pid).is_err()))
    }
}
