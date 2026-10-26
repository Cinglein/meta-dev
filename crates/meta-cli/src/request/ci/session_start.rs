use std::io::Write as _;
use std::path::PathBuf;

use super::session_id::SessionId;
use super::unix_seconds::UnixSeconds;
use crate::request::output::Listed;

#[derive(Debug)]
pub struct SessionStart(PathBuf);

impl SessionStart {
    const ROOT: &'static str = "meta-sessions";
    const FILE: &'static str = "started";

    pub fn for_session(session: &SessionId) -> Self {
        Self(std::env::temp_dir().join(Self::ROOT).join(session.file_name()).join(Self::FILE))
    }

    pub fn record(&self) -> Result<(), Listed> {
        self.written().map_err(|error| {
            Listed::new(format!("CI session start {self} unwritable: {error}; fix it or ask the owner"))
        })
    }

    fn written(&self) -> std::io::Result<()> {
        self.0.parent().map_or(Ok(()), std::fs::create_dir_all)?;
        match std::fs::File::create_new(&self.0) {
            Ok(mut file) => write!(file, "{}", UnixSeconds::now()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            Err(error) => Err(error),
        }
    }

    pub fn read(&self) -> Option<UnixSeconds> {
        std::fs::read_to_string(&self.0).ok()?.parse().ok()
    }
}

impl std::fmt::Display for SessionStart {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0.display())
    }
}
