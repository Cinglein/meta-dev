mod failed_runs;
mod killed_processes;

pub use failed_runs::FailedRuns;
pub use killed_processes::{KilledProcess, KilledProcesses};

use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::budget::AvailableGib;
use crate::failure_set::FailureSet;
use crate::report::Report;
use crate::site::Site;

#[derive(Debug, Error)]
pub enum Failure {
    #[error("{0}")]
    Usage(String),
    #[error("unknown flag {flag}; known: {known}")]
    UnknownFlag { flag: String, known: &'static str },
    #[error("no cargo workspace above the tooling")]
    NoWorkspace,
    #[error(
        "no merge base with the [branches] base of tooling/policy.toml; run `git fetch origin`"
    )]
    NoMergeBase,
    #[error("{0} must be set")]
    Unset(&'static str),
    #[error("{0} failed")]
    Command(String),
    #[error("{command} failed:\n{printed}")]
    Printed { command: String, printed: String },
    #[error("{printed}")]
    Found { printed: String, found: FailureSet },
    #[error("{0}")]
    Runs(FailedRuns),
    #[error("gh failed: {0}")]
    GitHub(String),
    #[error("cargo metadata is not the shape expected: {0}")]
    Metadata(#[from] serde_json::Error),
    #[error("mutant list is not the shape expected: {0}")]
    MutantList(serde_json::Error),
    #[error("{site}: cargo timing report is not the shape expected: {error}")]
    TimingReport {
        site: Site,
        error: serde_json::Error,
    },
    #[error("refused: {0}")]
    Refused(String),
    #[error(
        "underpriced in [mutant_millis]: {0}; regenerate it with mutant-costs over recent CI mutants artifacts"
    )]
    Underpriced(String),
    #[error("refused: another mutants run holds {}", .0.display())]
    LockHeld(PathBuf),
    #[error("refused: {} holds an earlier build; remove it for a clean build", .0.display())]
    EarlierBuild(PathBuf),
    #[error("refused: {0} is already running")]
    AlreadyRunning(&'static str),
    #[error("refused: {available} free, {granted} granted")]
    MemoryShort {
        available: AvailableGib,
        granted: AvailableGib,
    },
    #[error("refused: {budget} budget enforced, killed {killed}")]
    BudgetEnforced {
        budget: AvailableGib,
        killed: KilledProcesses,
    },
    #[error("refused: past its deadline, killed {killed}, own process groups included")]
    PastDeadline { killed: KilledProcesses },
    #[error("{site}: {error}")]
    Io { site: Site, error: io::Error },
    #[error("{site}: {error}")]
    Policy { site: Site, error: toml::de::Error },
    #[error("{site}: {error}")]
    Parse { site: Site, error: syn::Error },
    #[error("{}", .0.iter().map(ToString::to_string).collect::<Vec<String>>().join("\n"))]
    Lint(Vec<Report>),
}

impl Failure {
    #[must_use]
    pub fn found(&self) -> FailureSet {
        match self {
            Failure::Found { found, .. } => found.clone(),
            Failure::Lint(reports) => {
                FailureSet::lint(reports.iter().flat_map(Report::files).collect())
            }
            _ => FailureSet::default(),
        }
    }

    #[must_use]
    pub fn reading(self, read: impl FnOnce(&str) -> Option<Failure>) -> Failure {
        match self {
            Failure::Printed { command, printed } => {
                read(&printed).unwrap_or(Failure::Printed { command, printed })
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::{Failure, FailureSet, Report, Site};
    use crate::violation::Violation;

    const FILE: &str = "a.rs";

    #[test]
    fn a_lint_failure_reports_the_files_of_its_violations_and_another_failure_reports_none() {
        let report = Report::new(
            "slip",
            vec![Violation::new(Site::File(PathBuf::from(FILE)), FILE)],
        );
        let lint = FailureSet::lint(BTreeSet::from([PathBuf::from(FILE)]));
        assert_eq!(Failure::Lint(vec![report]).found(), lint);
        assert!(Failure::NoWorkspace.found().is_empty());
    }
}
