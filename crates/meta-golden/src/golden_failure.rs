use std::io;
use std::path::{Path, PathBuf};

use meta_run::Unread;
use thiserror::Error;

use crate::argv::Argv;
use crate::difference::Difference;

#[derive(Debug, Error)]
pub enum GoldenFailure {
    #[error("{}: {source}", path.display())]
    Unreadable { path: PathBuf, source: io::Error },
    #[error("{}: not a recording: {reason}", path.display())]
    Malformed { path: PathBuf, reason: String },
    #[error("{}: {difference}; run `cargo xtask goldens` and review the diff", path.display())]
    Differs { path: PathBuf, difference: Difference },
    #[error("{}: a setup step failed: {unread}", scenario.display())]
    Unbuilt { scenario: PathBuf, unread: Unread },
    #[error("no recording answers {0}")]
    Unanswered(Argv),
    #[error("recordings ran out for {asked}: all {answers_len} were answered")]
    Exhausted { asked: Argv, answers_len: usize },
    #[error("goldens are not blessed in CI; run `cargo xtask goldens` locally and commit the diff")]
    BlessedInCi,
}

impl GoldenFailure {
    #[must_use]
    pub fn differing(&self) -> Option<&Path> {
        match self {
            GoldenFailure::Differs { path, .. } => Some(path),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{Difference, GoldenFailure};

    const PATH: &str = "a.golden";

    #[test]
    fn only_a_differing_golden_names_its_path() {
        let difference = Difference::between("a", "b").unwrap();
        let differs = GoldenFailure::Differs { path: PathBuf::from(PATH), difference };
        assert_eq!(differs.differing(), Some(Path::new(PATH)));
        assert!(GoldenFailure::BlessedInCi.differing().is_none());
    }
}
