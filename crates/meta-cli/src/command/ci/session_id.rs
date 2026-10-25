use std::fmt;
use std::io::Write as _;

use meta_schema::Actor;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    pub fn announce(&self) -> Option<Actor> {
        let id: String = self.0.chars().filter(char::is_ascii_alphanumeric).collect();
        let actor: Actor = format!("claude-{}", id.to_ascii_lowercase()).parse().ok()?;
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

impl fmt::Display for SessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
