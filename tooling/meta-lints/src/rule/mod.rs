mod crate_graph;
mod findings;
mod report;

pub use crate_graph::{CrateGraph, GraphLimits, GraphReport};
pub use findings::Findings;

use strum::{EnumIter, IntoStaticStr};

use crate::lint_config::LintConfig;
use crate::rule::report::Report;
use crate::source_file::SourceFile;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Rule {
    CrateGraph,
}

impl Rule {
    pub fn report(self, files: &[SourceFile], config: &LintConfig) -> Report {
        match self {
            Rule::CrateGraph => CrateGraph::report(files, config.crate_graph()),
        }
    }
}
