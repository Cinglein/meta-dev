mod revision;

pub use revision::{ChangedPath, Revision};

use std::path::Path;

use crate::baseline::Baseline;
use crate::drift::Drift;
use crate::failure::Failure;
use crate::site::Site;

#[derive(Clone, Debug)]
pub struct Prior {
    baseline: Baseline,
}

impl Prior {
    pub fn read_at(revision: &Revision, baseline: &Path) -> Result<Prior, Failure> {
        let text = revision.file(baseline)?;
        let baseline = text.map(|text| Baseline::parse(&text, Site::of_path(baseline))).transpose()?;
        Ok(Prior { baseline: baseline.unwrap_or_default() })
    }

    pub fn admit(&self, baseline: &Baseline) -> Result<(), Failure> {
        let raised: Vec<Drift> = baseline.drifts(&self.baseline).into_iter().filter(Drift::grew).collect();
        if raised.is_empty() { Ok(()) } else { Err(Failure::Raised(raised)) }
    }
}
