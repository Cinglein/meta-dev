mod gc_report;

use std::process::Command;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::request::local::Sweep;
use crate::request::project::GcMode;
use crate::request::timed_command::{Finished, TimedCommand};
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
    pub fn supervised(command: Command, deadline: Instant, mode: GcMode) -> Self {
        mode.sweep().map_or(Self::Disabled, |sweep| {
            TimedCommand::new(command, deadline).finished().map_or_else(
                |reason| Self::Unfinished(GcReport::shown(reason)),
                |finished| Self::reported(finished, sweep),
            )
        })
    }

    fn reported(finished: Finished, sweep: Sweep) -> Self {
        match (finished.succeeded(), sweep) {
            (Ok(report), Sweep::Planned) => Self::Planned(GcReport::shown(report)),
            (Ok(report), Sweep::Done) => Self::Freed(GcReport::shown(report)),
            (Err(complaint), _) => Self::Failed(GcReport::shown(complaint)),
        }
    }
}
