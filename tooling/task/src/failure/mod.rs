mod failed_runs;
mod killed_processes;

pub use failed_runs::FailedRuns;
pub use killed_processes::{KilledProcess, KilledProcesses};

use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::budget::AvailableGib;
use crate::report::Report;
use crate::site::Site;

#[derive(Debug, Error)]
pub enum Failure {
    #[error("{0}")]
    Usage(String),
    #[error("unknown flag {flag}; the only flag is {known}")]
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
    #[error("{0}")]
    Runs(FailedRuns),
    #[error("gh failed: {0}")]
    GitHub(String),
    #[error("cargo metadata is not the shape expected: {0}")]
    Metadata(#[from] serde_json::Error),
    #[error("mutant list is not the shape expected: {0}")]
    MutantList(serde_json::Error),
    #[error("refused: {0}")]
    Refused(String),
    #[error(
        "underpriced in [mutant_millis]: {0}; regenerate it with mutant-costs over recent CI mutants artifacts"
    )]
    Underpriced(String),
    #[error("refused: another mutants run holds {}", .0.display())]
    LockHeld(PathBuf),
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
    #[error("{site}: {error}")]
    Io { site: Site, error: io::Error },
    #[error("{site}: {error}")]
    Policy { site: Site, error: toml::de::Error },
    #[error("{site}: {error}")]
    Parse { site: Site, error: syn::Error },
    #[error("{}", .0.iter().map(ToString::to_string).collect::<Vec<String>>().join("\n"))]
    Lint(Vec<Report>),
}
