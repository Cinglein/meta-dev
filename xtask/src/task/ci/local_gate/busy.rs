use meta_door::Door;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug)]
pub struct Busy {
    lock: PathBuf,
    waited: Duration,
}

impl Busy {
    pub fn new(lock: PathBuf, waited: Duration) -> Busy {
        Busy { lock, waited }
    }
}

impl fmt::Display for Busy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let holder = Door::read(&self.lock).unwrap_or_default();
        write!(formatter, "gate lock {} held by {holder}; gave up after {:?}", self.lock.display(), self.waited)
    }
}
