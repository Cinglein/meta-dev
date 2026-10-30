use std::path::PathBuf;

use super::session_id::SessionId;
use super::start_moment::StartMoment;
use crate::request::environment::Environment;
use crate::request::output::Listed;

#[derive(Debug)]
pub struct SessionStart(PathBuf);

impl SessionStart {
    const ROOT: &'static str = "meta-sessions";
    const STARTED: &'static str = "started";
    const WRITES: &'static str = "writes";

    pub fn for_session(session: &SessionId, environment: &Environment) -> Self {
        Self(environment.temporary().join(Self::ROOT).join(session.file_name()))
    }

    pub fn open(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.0)?;
        let started = self.0.join(Self::STARTED);
        std::fs::exists(&started)?
            .then_some(())
            .map_or_else(|| std::fs::write(&started, StartMoment::now().to_string()), Ok)
    }

    pub fn record(&self) -> Result<(), Listed> {
        self.open().and_then(|()| std::fs::File::create(self.0.join(Self::WRITES)).map(drop)).map_err(|error| {
            Listed::new(format!("CI session start {self} unwritable: {error}; fix it or ask the owner"))
        })
    }

    pub fn read(&self) -> Option<StartMoment> {
        std::fs::exists(self.0.join(Self::WRITES)).ok().filter(|writes| *writes)?;
        std::fs::read_to_string(self.0.join(Self::STARTED)).ok()?.parse().ok()
    }
}

impl std::fmt::Display for SessionStart {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0.display())
    }
}
