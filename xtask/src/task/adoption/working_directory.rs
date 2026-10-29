use std::convert::Infallible;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkingDirectory(PathBuf);

impl WorkingDirectory {
    const HERE: &str = ".";
    const CURRENT: &str = "./";
    const SEPARATOR: char = '/';

    pub fn within(repository: &Path, workspace: &Path) -> WorkingDirectory {
        WorkingDirectory(workspace.strip_prefix(repository).map(Path::to_path_buf).unwrap_or_default())
    }
}

impl FromStr for WorkingDirectory {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<WorkingDirectory, Infallible> {
        let text = text.trim().trim_matches(['"', '\'']);
        let text = text.strip_prefix(Self::CURRENT).unwrap_or(text).trim_end_matches(Self::SEPARATOR);
        Ok(WorkingDirectory(PathBuf::from(if text == Self::HERE { "" } else { text })))
    }
}

impl fmt::Display for WorkingDirectory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.as_os_str().is_empty() {
            formatter.write_str(Self::HERE)
        } else {
            write!(formatter, "{}", self.0.display())
        }
    }
}
