mod gc_report;

use std::time::Instant;

use meta_run::{Ran, Run};
use serde::{Deserialize, Serialize};

use crate::request::local::Sweep;
use crate::request::project::GcMode;
pub use gc_report::GcReport;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GcHook {
    Pending,
    Disabled,
    Planned(GcReport),
    Freed(GcReport),
    Failed(GcReport),
    Unfinished(GcReport),
}

impl GcHook {
    pub fn supervised(run: Run, deadline: Instant, mode: GcMode) -> Self {
        mode.sweep().map_or(Self::Disabled, |sweep| {
            run.ran(deadline)
                .map_or_else(|reason| Self::Unfinished(GcReport::shown(reason)), |ran| Self::reported(ran, sweep))
        })
    }

    fn reported(ran: Ran, sweep: Sweep) -> Self {
        match (ran.succeeded(), sweep) {
            (Ok(report), Sweep::Planned) => Self::Planned(GcReport::shown(report)),
            (Ok(report), Sweep::Done) => Self::Freed(GcReport::shown(report)),
            (Err(complaint), _) => Self::Failed(GcReport::shown(complaint)),
        }
    }
}
