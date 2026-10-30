use std::path::{Path, PathBuf};

use meta_schema::{Id, Task};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskWorktree {
    task: Id<Task>,
    path: PathBuf,
}

impl TaskWorktree {
    pub fn enclosing(directory: &Path) -> Option<Self> {
        directory.ancestors().find_map(|folder| {
            let task = folder.file_name()?.to_str()?.parse().ok()?;
            Some(Self { task, path: folder.to_path_buf() })
        })
    }

    pub fn canonical(self) -> Self {
        let path = std::fs::canonicalize(&self.path).unwrap_or(self.path);
        Self { path, ..self }
    }

    pub fn task(&self) -> Id<Task> {
        self.task
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
