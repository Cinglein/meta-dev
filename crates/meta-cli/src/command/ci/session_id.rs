use std::fmt;
use std::io::Write as _;
use std::path::PathBuf;
use std::str::FromStr;

use meta_schema::Actor;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    pub fn file_name(&self) -> PathBuf {
        PathBuf::from(self.0.chars().filter(char::is_ascii_alphanumeric).collect::<String>())
    }

    pub fn announce(&self) -> Option<Actor> {
        let actor: Actor =
            format!("claude-{}", self.file_name().display().to_string().to_ascii_lowercase()).parse().ok()?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(std::env::var_os("CLAUDE_ENV_FILE")?)
            .ok()?
            .write_all(format!("export META_ACTOR={actor}\n").as_bytes())
            .ok()?;
        Some(actor)
    }
}

impl FromStr for SessionId {
    type Err = std::convert::Infallible;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Ok(Self(id.to_owned()))
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
