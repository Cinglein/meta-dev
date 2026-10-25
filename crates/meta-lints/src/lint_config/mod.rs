use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde::Deserialize;

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
