use std::time::Duration;

use chrono::Utc;
use meta_log::LogStore;
use meta_pull_query::LiveQuery;
use meta_severity::Scales;
use meta_state::State;
use meta_status::{Flags, Github, SeverityTally, Status};

use crate::request::cli_failure::CliFailure;
use crate::request::output::{Listed, Output};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusView {
    Tree,
    Flags,
}

impl StatusView {
    pub fn answer(
        self,
        log: &impl LogStore,
        scales: &Scales,
        ci_run_within: Duration,
        github: impl FnOnce(&LiveQuery) -> Github,
    ) -> Result<Output, CliFailure> {
        match self {
            Self::Tree => {
                let snapshot = log.read()?;
                let state = State::fold(snapshot.records());
                let github = github(&Status::query(&state));
                let status = Status::new(&state, &github, Utc::now().into(), ci_run_within);
                let tally = SeverityTally::new(&state, scales);
                Ok(Output::Listed(Listed::new(status).then(tally).then(snapshot)))
            }
            Self::Flags => Self::current_flags(log, ci_run_within, github).map(|flags| Self::flagged(&flags)),
        }
    }

    fn flagged(flags: &Flags) -> Output {
        flags.raised().map_or_else(|| Output::Listed(Listed::new("")), |raised| Output::Flagged(Listed::new(raised)))
    }

    pub fn current_flags(
        log: &impl LogStore,
        ci_run_within: Duration,
        github: impl FnOnce(&LiveQuery) -> Github,
    ) -> Result<Flags, CliFailure> {
        let snapshot = log.read()?;
        let state = State::fold(snapshot.records());
        let github = github(&Status::query(&state));
        Ok(Status::new(&state, &github, Utc::now().into(), ci_run_within).flags())
    }
}
