use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::iter;
use std::path::{Component, Path, PathBuf};
use std::process;
use std::thread;
use std::time::{Duration, Instant};

use task::{Millis, Site};

use crate::task::ci::local_gate::Busy;
use crate::task::failure::Failure;

#[derive(Debug)]
pub struct GateLock(File);

impl GateLock {
    const PAUSE: Duration = Duration::from_millis(250);
    const TARGETS: &str = "targets";
    const LOCK: &str = "lock";

    pub fn target(parent: &Path, workspace: &Path) -> PathBuf {
        workspace
            .components()
            .filter(|part| matches!(part, Component::Normal(_)))
            .fold(parent.join(Self::TARGETS), |target, _| target)
    }

    pub fn guarding(target: &Path, wait: Millis) -> Result<GateLock, Failure> {
        fs::create_dir_all(target).map_err(|error| task::Failure::Io { site: Site::File(target.into()), error })?;
        Self::acquire(&target.with_extension(Self::LOCK), wait.duration())
    }

    fn acquire(path: &Path, wait: Duration) -> Result<GateLock, Failure> {
        let io = |error| Failure::from(task::Failure::Io { site: Site::File(path.into()), error });
        let mut file = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(path).map_err(io)?;
        let deadline = Instant::now() + wait;
        iter::successors(Some(()), |()| {
            (Instant::now() < deadline).then(|| {
                thread::sleep(Self::PAUSE);
            })
        })
        .any(|()| file.try_lock().is_ok())
        .then_some(())
        .ok_or_else(|| Failure::Busy(Busy::new(path.to_path_buf(), wait)))?;
        file.set_len(0).map_err(io)?;
        write!(file, "pid {}", process::id()).map_err(io)?;
        Ok(GateLock(file))
    }
}

impl Drop for GateLock {
    fn drop(&mut self) {
        self.0.unlock().ok();
    }
}

#[cfg(test)]
mod tests {
    use std::process;
    use std::time::Duration;

    use task::Millis;

    use super::GateLock;
    use crate::task::fixture::Fixture;

    const LOCK: &str = "gate.lock";
    const SCOPE: &str = "consumer";

    #[test]
    fn a_held_gate_lock_makes_the_next_gate_give_up_naming_the_holder() {
        let fixture = Fixture::new();
        let held = GateLock::acquire(&fixture.path(LOCK), Duration::ZERO).unwrap();
        let refused = GateLock::acquire(&fixture.path(LOCK), Duration::ZERO).map_err(|failure| failure.to_string());
        assert!(refused.err().is_some_and(|message| message.contains(&process::id().to_string())));
        drop(held);
        assert!(GateLock::acquire(&fixture.path(LOCK), Duration::ZERO).is_ok());
    }

    #[test]
    fn each_workspace_gets_its_own_gate_target_outside_it() {
        let fixture = Fixture::new();
        let target = GateLock::target(&fixture.beside(), fixture.root());
        let other = GateLock::target(&fixture.beside(), &fixture.path(SCOPE));
        assert!(target.starts_with(fixture.beside()) && !target.starts_with(fixture.root()));
        assert!(target != other && target == GateLock::target(&fixture.beside(), fixture.root()));
        assert!(GateLock::guarding(&target, Millis::default()).is_ok() && target.is_dir());
    }
}
