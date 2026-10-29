use std::fmt;

use crate::task::adoption::required_tool::RequiredTool;
use crate::task::adoption::skip::Skip;
use crate::task::adoption::working_directory::WorkingDirectory;

#[derive(Debug)]
pub enum Gap {
    Uninstalled(RequiredTool),
    Absent(RequiredTool, WorkingDirectory),
    Skippable(RequiredTool, String, Skip),
    TurnedOff(RequiredTool),
}

impl fmt::Display for Gap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Gap::Uninstalled(tool) => write!(formatter, "{tool} is not installed; fix: {}", tool.fix()),
            Gap::Absent(tool, directory) => {
                write!(formatter, "{tool}: no workflow step runs it in {directory}; fix: {}", tool.fix())
            }
            Gap::Skippable(tool, place, skip) => {
                write!(formatter, "{tool}: {place} can skip it through {skip}; fix: remove that so it always runs")
            }
            Gap::TurnedOff(tool) => write!(formatter, "{tool}: tooling/policy.toml turns it off; fix: {}", tool.fix()),
        }
    }
}
