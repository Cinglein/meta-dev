use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::rule::GraphLimits;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LintConfig {
    baseline: PathBuf,
    #[serde(default)]
    exclude: Vec<PathBuf>,
    #[serde(default)]
    crate_graph: Option<GraphLimits>,
}

impl LintConfig {
    pub fn baseline(&self) -> &Path {
        &self.baseline
    }

    pub fn crate_graph(&self) -> Option<GraphLimits> {
        self.crate_graph
    }

    pub fn excludes(&self, directory: &OsStr) -> bool {
        self.exclude.iter().any(|excluded| excluded == directory)
    }
}
