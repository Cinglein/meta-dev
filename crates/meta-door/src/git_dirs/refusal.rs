use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Refusal {
    Inside { path: PathBuf, git: PathBuf },
    Unknown { directory: PathBuf, said: String },
    Unrun { directory: PathBuf, failure: io::Error },
    Escaped { path: PathBuf, area: PathBuf },
    Dangling { path: PathBuf },
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Inside { path, git } => write!(
                formatter,
                "{} is inside the git dir {}; use git for refs, MetaStore for meta files",
                path.display(),
                git.display()
            ),
            Refusal::Unknown { directory, said } => {
                write!(formatter, "{}: git cannot tell its git dir: {}", directory.display(), said.trim())
            }
            Refusal::Unrun { directory, failure } => {
                write!(formatter, "{}: git did not run: {failure}", directory.display())
            }
            Refusal::Escaped { path, area } => {
                write!(
                    formatter,
                    "{} resolves outside {}; meta files stay in their area",
                    path.display(),
                    area.display()
                )
            }
            Refusal::Dangling { path } => {
                write!(formatter, "{} is a link to nothing; the door does not follow it", path.display())
            }
        }
    }
}

impl std::error::Error for Refusal {}
