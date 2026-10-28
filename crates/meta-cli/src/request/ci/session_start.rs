use std::io::ErrorKind;
use std::path::PathBuf;

use meta_door::{Change, Door, Opening};

use super::session_id::SessionId;
use super::start_moment::StartMoment;
use crate::request::output::Listed;

#[derive(Debug)]
pub struct SessionStart(PathBuf);

impl SessionStart {
    const ROOT: &'static str = "meta-sessions";
    const STARTED: &'static str = "started";
    const WRITES: &'static str = "writes";

    pub fn for_session(session: &SessionId) -> Self {
        Self(std::env::temp_dir().join(Self::ROOT).join(session.file_name()))
    }

    pub fn open(&self) -> std::io::Result<()> {
        Door::changed(&self.0, Change::DirAll)?;
        let started = self.0.join(Self::STARTED);
        Door::described(&started).map(drop).or_else(|failure| match failure.kind() {
            ErrorKind::NotFound => Door::write(&started, StartMoment::now().to_string()),
            _ => Err(failure),
        })
    }

    pub fn record(&self) -> Result<(), Listed> {
        self.open().and_then(|()| Door::opened(self.0.join(Self::WRITES), Opening::Truncate).map(drop)).map_err(
            |error| Listed::new(format!("CI session start {self} unwritable: {error}; fix it or ask the owner")),
        )
    }

    pub fn read(&self) -> Option<StartMoment> {
        Door::described(self.0.join(Self::WRITES)).ok()?;
        Door::read(self.0.join(Self::STARTED)).ok()?.parse().ok()
    }
}

impl std::fmt::Display for SessionStart {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0.display())
    }
}
