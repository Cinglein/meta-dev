use std::fmt;
use std::io::Write as _;
use std::path::PathBuf;
use std::str::FromStr;

use meta_door::{Door, Opening};
use meta_schema::Actor;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    pub fn file_name(&self) -> PathBuf {
        PathBuf::from(self.0.chars().filter(char::is_ascii_alphanumeric).collect::<String>())
    }

    pub const VARIABLE: &'static str = "META_SESSION";

    pub fn announce(&self, actor: Option<Actor>) -> Option<Actor> {
        let actor = actor.or_else(|| {
            format!("claude-{}", self.file_name().display().to_string().to_ascii_lowercase()).parse().ok()
        })?;
        Door::opened(std::env::var_os("CLAUDE_ENV_FILE")?, Opening::Append)
            .ok()?
            .write_all(format!("export META_ACTOR={actor}\nexport {}={}\n", Self::VARIABLE, self.0).as_bytes())
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
