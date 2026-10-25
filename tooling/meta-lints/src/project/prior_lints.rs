use std::path::PathBuf;

use serde::Deserialize;

use crate::baseline::Baseline;
use crate::failure::Failure;
use crate::prior::{Prior, Revision};
use crate::site::Site;

#[derive(Clone, Debug, Deserialize)]
pub struct PriorLints {
    baseline: PathBuf,
}

impl PriorLints {
    pub fn new(baseline: PathBuf) -> PriorLints {
        PriorLints { baseline }
    }

    pub fn prior(&self, revision: &Revision) -> Result<Prior, Failure> {
        let text = revision.file(&self.baseline)?;
        let baseline = text.map(|text| Baseline::parse(&text, Site::of_path(&self.baseline))).transpose()?;
        Ok(Prior::new(baseline.unwrap_or_default()))
    }
}
