use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::loosening::Loosening;
use crate::path_prefix::PathPrefix;
use crate::region::Region;
use crate::rule::OwnershipRule;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LintConfig {
    baseline: PathBuf,
    #[serde(default)]
    exclude: Vec<PathBuf>,
    #[serde(default)]
    product: Vec<PathPrefix>,
    #[serde(default)]
    boundary: Vec<PathPrefix>,
    #[serde(default)]
    ownership: Vec<OwnershipRule>,
}

impl LintConfig {
    const ALWAYS_EXCLUDED: &str = "target";

    pub fn baseline(&self) -> &Path {
        &self.baseline
    }

    pub fn ownership(&self) -> &[OwnershipRule] {
        &self.ownership
    }

    pub fn excludes(&self, directory: &OsStr) -> bool {
        directory == Self::ALWAYS_EXCLUDED || self.exclude.iter().any(|excluded| excluded == directory)
    }

    pub fn loosened_from(&self, prior: &LintConfig) -> Vec<Loosening> {
        let moved = (self.baseline != prior.baseline)
            .then(|| Loosening::BaselineMoved { from: prior.baseline.clone(), to: self.baseline.clone() });
        let excluded = self.exclude.iter().filter(|directory| !prior.exclude.contains(directory)).cloned();
        let dropped = prior.product.iter().filter(|prefix| !self.product.contains(prefix)).cloned();
        let added = self.boundary.iter().filter(|prefix| !prior.boundary.contains(prefix)).cloned();
        moved
            .into_iter()
            .chain(excluded.map(Loosening::Excluded))
            .chain(dropped.map(Loosening::ProductDropped))
            .chain(added.map(Loosening::BoundaryAdded))
            .collect()
    }

    pub fn region(&self, path: &str) -> Region {
        if self.boundary.iter().any(|prefix| prefix.covers(path)) {
            Region::Boundary
        } else if self.product.iter().any(|prefix| prefix.covers(path)) {
            Region::Product
        } else {
            Region::Support
        }
    }
}
