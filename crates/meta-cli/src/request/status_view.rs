use chrono::Utc;
use meta_log::LogStore;
use meta_pull_query::LiveQuery;
use meta_state::State;
use meta_status::{Github, Status};

use crate::request::cli_failure::CliFailure;
use crate::request::output::{Listed, Output};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusView {
    Tree,
    Flags,
}

impl StatusView {
    pub fn answer(self, log: &impl LogStore, github: impl FnOnce(&LiveQuery) -> Github) -> Result<Output, CliFailure> {
        let snapshot = log.read()?;
        let state = State::fold(snapshot.records());
        let github = github(&Status::query(&state));
        let status = Status::new(&state, &github, Utc::now().into());
        let flags = status.flags();
        Ok(match self {
            Self::Tree => Output::Listed(Listed::new(status).then(snapshot)),
            Self::Flags => flags
                .raised()
                .map_or_else(|| Output::Listed(Listed::new("")), |raised| Output::Flagged(Listed::new(raised))),
        })
    }
}
