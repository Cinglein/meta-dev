use std::fmt;

use crate::task::adoption::required_tool::RequiredTool;
use crate::task::adoption::skip::Skip;
use crate::task::adoption::working_directory::WorkingDirectory;
use crate::task::failure::Failure;

#[derive(Debug)]
pub enum Gap {
    Uninstalled(RequiredTool),
    Absent(RequiredTool, WorkingDirectory),
    Skippable(RequiredTool, String, Skip),
    TurnedOff(RequiredTool),
    Unlisted(RequiredTool, WorkingDirectory),
    Alerted(RequiredTool, String),
    Unreadable(RequiredTool, Box<Failure>),
}

impl fmt::Display for Gap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Gap::Uninstalled(tool) => write!(formatter, "{tool} is not installed; fix: {}", tool.fix()),
            Gap::Absent(tool, directory) => {
                write!(formatter, "{tool}: no workflow step runs it in {directory}; fix: {}", tool.fix())
            }
            Gap::Skippable(tool, place, skip) => {
                let trigger = tool.trigger();
                write!(
                    formatter,
                    "{tool}: {place} can skip it through {skip}; fix: drop that so it always runs on {trigger}"
                )
            }
            Gap::TurnedOff(tool) => write!(formatter, "{tool}: tooling/policy.toml turns it off; fix: {}", tool.fix()),
            Gap::Unlisted(tool, directory) => {
                write!(formatter, "{tool}: .github/dependabot.yml does not list {directory}; fix: {}", tool.fix())
            }
            Gap::Alerted(tool, count) => {
                write!(formatter, "{tool}: {} open alerts; fix: {}", count.trim(), tool.fix())
            }
            Gap::Unreadable(tool, failure) => write!(formatter, "{tool}: {failure}; fix: {}", tool.fix()),
        }
    }
}
