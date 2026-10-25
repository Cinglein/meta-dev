mod revision;

pub use revision::Revision;

use std::path::Path;

use crate::baseline::Baseline;
use crate::drift::Drift;
use crate::failure::Failure;
use crate::lint_config::LintConfig;
use crate::loosening::Loosening;
use crate::site::Site;

#[derive(Clone, Debug)]
pub struct Prior {
    config: LintConfig,
    baseline: Baseline,
}

impl Prior {
    pub fn read_at(revision: &Revision, config: LintConfig) -> Result<Prior, Failure> {
        let baseline = revision
            .file(config.baseline())?
            .map(|text| Baseline::parse(&text, Site::of_path(Path::new(config.baseline()))))
            .transpose()?
            .unwrap_or_default();
        Ok(Prior { config, baseline })
    }

    pub fn admit(&self, config: &LintConfig, baseline: &Baseline) -> Result<(), Failure> {
        let loosenings: Vec<Loosening> = config
            .loosened_from(&self.config)
            .into_iter()
            .chain(baseline.drifts(&self.baseline).into_iter().filter(Drift::grew).map(Loosening::Raised))
            .collect();
        if loosenings.is_empty() { Ok(()) } else { Err(Failure::Loosened(loosenings)) }
    }
}
