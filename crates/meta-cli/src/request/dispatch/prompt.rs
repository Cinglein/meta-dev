use std::path::PathBuf;

use meta_log::LogStore;
use meta_state::State;
use serde::Deserialize;

use super::{Brief, LeaseLine};
use crate::request::output::Listed;
use crate::request::scale_policy::ScalePolicy;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Prompt(String);

impl Prompt {
    pub fn project(&self) -> Option<PathBuf> {
        self.0.lines().find_map(|line| line.strip_prefix(Brief::PROJECT)).map(PathBuf::from)
    }

    pub fn admitted(&self, log: &impl LogStore) -> Result<(), Listed> {
        let line = self.0.lines().find(|line| line.starts_with(LeaseLine::PREFIX));
        let lease: LeaseLine = line.ok_or_else(|| Listed::new("no lease line in the prompt"))?.parse()?;
        let issuers = ScalePolicy::embedded_section(ScalePolicy::dispatch).map_err(Listed::new)?;
        lease.live_in(&State::fold(log.read().map_err(Listed::new)?.records()), issuers)
    }
}
