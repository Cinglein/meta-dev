use std::fmt;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use meta_schema::{Actor, Id, Task};
use serde::Deserialize;

use super::task_worktree::TaskWorktree;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    pub fn file_name(&self) -> PathBuf {
        PathBuf::from(self.0.chars().filter(char::is_ascii_alphanumeric).collect::<String>())
    }

    pub const VARIABLE: &'static str = "META_SESSION";

    pub fn of_task(task: Id<Task>) -> Self {
        Self(task.to_string())
    }

    pub fn chosen(variable: Option<String>, directory: &Path) -> Option<Self> {
        let claimed = || TaskWorktree::enclosing(directory).map(|worktree| worktree.task());
        variable.map(Self).or_else(|| claimed().map(Self::of_task))
    }

    pub fn announce(&self, actor: Option<Actor>) -> Option<Actor> {
        let actor = actor.or_else(|| {
            format!("claude-{}", self.file_name().display().to_string().to_ascii_lowercase()).parse().ok()
        })?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(std::env::var_os("CLAUDE_ENV_FILE")?)
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

#[cfg(test)]
mod tests {
    use super::{Path, SessionId};

    #[test]
    fn the_variable_wins_else_the_enclosing_task_worktree_names_the_session_else_none() {
        let (said, task) = ("s", "task-53c6698cb2d5ea01");
        let inside = Path::new("/trees/task-53c6698cb2d5ea01/src");
        let chosen =
            |variable: Option<&str>, at| SessionId::chosen(variable.map(str::to_owned), at).map(|id| id.to_string());
        assert_eq!(chosen(Some(said), inside).as_deref(), Some(said));
        assert_eq!(chosen(None, inside).as_deref(), Some(task));
        assert_eq!(chosen(None, Path::new("/trees/clone")), None);
    }
}
