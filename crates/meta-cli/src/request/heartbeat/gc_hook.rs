mod gc_report;

use serde::{Deserialize, Serialize};

use crate::request::local::LocalClone;
use crate::request::project::{GcMode, Project};
pub use gc_report::GcReport;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GcHook {
    Planned(GcReport),
    Freed(GcReport),
    Failed(GcReport),
}

impl GcHook {
    pub fn run(project: &Project, clone: &LocalClone, mode: GcMode) -> Self {
        match (clone.collected(project, mode.sweep()), mode) {
            (Ok(swept), GcMode::DryRun) => Self::Planned(GcReport::shown(swept)),
            (Ok(swept), GcMode::Free) => Self::Freed(GcReport::shown(swept)),
            (Err(failure), _) => Self::Failed(GcReport::shown(failure)),
        }
    }
}
