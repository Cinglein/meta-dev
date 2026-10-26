use chrono::Utc;
use meta_log::LogStore;
use meta_pull_query::LiveQuery;
use meta_state::State;
use meta_status::{Flags, Github, Status};

use crate::request::cli_failure::CliFailure;
use crate::request::output::{Listed, Output};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusView {
    Tree,
    Flags,
}

impl StatusView {
    pub fn answer(self, log: &impl LogStore, github: impl FnOnce(&LiveQuery) -> Github) -> Result<Output, CliFailure> {
        match self {
            Self::Tree => {
                let snapshot = log.read()?;
                let state = State::fold(snapshot.records());
                let github = github(&Status::query(&state));
                let status = Status::new(&state, &github, Utc::now().into());
                Ok(Output::Listed(Listed::new(status).then(snapshot)))
            }
            Self::Flags => Self::current_flags(log, github).map(|flags| Self::flagged(&flags)),
        }
    }

    fn flagged(flags: &Flags) -> Output {
        flags.raised().map_or_else(|| Output::Listed(Listed::new("")), |raised| Output::Flagged(Listed::new(raised)))
    }

    pub fn current_flags(log: &impl LogStore, github: impl FnOnce(&LiveQuery) -> Github) -> Result<Flags, CliFailure> {
        let snapshot = log.read()?;
        let state = State::fold(snapshot.records());
        let github = github(&Status::query(&state));
        Ok(Status::new(&state, &github, Utc::now().into()).flags())
    }
}
