use std::fmt;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct AgentRoot(PathBuf);

impl AgentRoot {
    const HOME: &'static str = "~";

    pub fn own(path: &Path) -> Self {
        Self(std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()))
    }

    pub fn anchored(self, checkout: &Path, home: &Path) -> Option<Self> {
        let mut parts = self.0.components();
        let joined = match parts.next().filter(|first| *first != Component::CurDir)? {
            Component::Normal(first) if first == Self::HOME => home.join(parts.as_path()),
            _ => checkout.join(&self.0),
        };
        let root = std::fs::canonicalize(&joined)
            .ok()
            .or_else(|| joined.components().all(|part| part != Component::ParentDir).then_some(joined))?;
        let covers = |guarded: &Path| Self::own(guarded).0.starts_with(&root);
        (root.parent().is_some() && !covers(home) && !covers(checkout)).then_some(Self(root))
    }

    pub fn holds(&self, path: &Path) -> Option<()> {
        path.starts_with(&self.0).then_some(())
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl fmt::Display for AgentRoot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.file_name().unwrap_or_default().to_string_lossy())
    }
}
