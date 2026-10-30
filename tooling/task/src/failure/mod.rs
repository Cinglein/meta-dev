mod failed_runs;
mod killed_processes;
mod printed;

pub use failed_runs::FailedRuns;
pub use killed_processes::{KilledProcess, KilledProcesses};
pub use printed::Printed;

use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::budget::AvailableGib;
use crate::check_name::CheckName;
use crate::failure_kind::FailureKind;
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
    Found { printed: Printed, found: FailureSet },
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
    pub fn found(&self, job: &CheckName) -> FailureSet {
        match self {
            Failure::Found { found, .. } => found.clone(),
            Failure::Lint(reports) => {
                let lint = FailureSet::lint(reports.iter().flat_map(Report::files).collect());
                let unplaced = FailureKind::Infra { job: job.clone() };
                if reports
                    .iter()
                    .any(|report| report.first_unplaced().is_some())
                {
                    lint.recording(unplaced)
                } else {
                    lint
                }
            }
            _ => FailureSet::default(),
        }
    }

    #[must_use]
    pub fn incomplete(self, job: CheckName) -> Failure {
        match self {
            Failure::Found { printed, found } => Failure::Found {
                printed,
                found: found.recording(FailureKind::Infra { job }),
            },
            other => other,
        }
    }

    #[must_use]
    pub fn reading(self, read: impl FnOnce(&str) -> Option<Failure>) -> Failure {
        match self {
            Failure::Printed { command, printed } => match read(&printed) {
                Some(Failure::Found {
                    printed: text,
                    found,
                }) => Failure::Found {
                    printed: Printed::Under {
                        command,
                        text: Box::new(text),
                    },
                    found,
                },
                _ => Failure::Printed { command, printed },
            },
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::{Failure, FailureKind, FailureSet, Printed, Report, Site};
    use crate::violation::Violation;

    const FILE: &str = "a.rs";
    const JOB: &str = "lint";

    const SHOWN: &str = "cmd failed:\nshown";

    impl Failure {
        fn linting(sites: Vec<Site>) -> FailureSet {
            let violations = sites
                .into_iter()
                .map(|site| Violation::new(site, FILE))
                .collect();
            Failure::Lint(vec![Report::new(FILE, violations)]).found(&JOB.into())
        }

        fn shown() -> Failure {
            Failure::Found {
                printed: Printed::Text("shown".into()),
                found: FailureSet::default(),
            }
        }
    }

    #[test]
    fn lint_files_are_named_and_an_unplaced_violation_adds_infra() {
        let file = Site::File(PathBuf::from(FILE));
        let infra = FailureKind::Infra { job: JOB.into() };
        assert_eq!(
            Failure::linting(vec![file.clone()]),
            FailureSet::lint(BTreeSet::from([PathBuf::from(FILE)]))
        );
        assert_eq!(
            Failure::linting(vec![file, Site::Workspace]).kinds().last(),
            Some(&infra)
        );
        assert!(Failure::NoWorkspace.found(&JOB.into()).is_empty());
    }

    #[test]
    fn a_printed_failure_gains_its_command_header_only_when_read() {
        let printed = || Failure::Printed {
            command: "cmd".into(),
            printed: FILE.into(),
        };
        assert_eq!(
            printed().reading(|_| Some(Failure::shown())).to_string(),
            SHOWN
        );
        assert_eq!(printed().reading(|_| None).to_string(), "cmd failed:\na.rs");
    }

    #[test]
    fn an_incomplete_found_set_gains_infra() {
        let kinds = Failure::shown().incomplete(JOB.into()).found(&JOB.into());
        assert_eq!(kinds.kinds(), [FailureKind::Infra { job: JOB.into() }]);
    }
}
