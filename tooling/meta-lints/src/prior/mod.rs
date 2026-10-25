mod revision;

pub use revision::{ChangedPath, Revision};

use crate::baseline::Baseline;
use crate::drift::Drift;
use crate::failure::Failure;

#[derive(Clone, Debug)]
pub struct Prior {
    baseline: Baseline,
}

impl Prior {
    pub fn new(baseline: Baseline) -> Prior {
        Prior { baseline }
    }

    pub fn admit(&self, baseline: &Baseline) -> Result<(), Failure> {
        let raised: Vec<Drift> = baseline.drifts(&self.baseline).into_iter().filter(Drift::grew).collect();
        if raised.is_empty() { Ok(()) } else { Err(Failure::Raised(raised)) }
    }
}
