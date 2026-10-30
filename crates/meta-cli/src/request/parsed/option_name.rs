use std::fmt;
use std::str::FromStr;

use crate::request::cli_failure::{CliFailure, Misuse};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionName {
    Source,
    Tag,
    Addresses,
    From,
    All,
    Division,
    Effort,
    Due,
    On,
    Flags,
    Worktree,
    DryRun,
    Severity,
    Scenario,
    Actor,
    Scratch,
    Instruments,
    Serves,
    CausedBy,
    Project,
    Root,
    Row,
    Span,
    Gate,
}

impl FromStr for OptionName {
    type Err = CliFailure;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "--source" => Ok(Self::Source),
            "--tag" => Ok(Self::Tag),
            "--addresses" => Ok(Self::Addresses),
            "--from" => Ok(Self::From),
            "--all" => Ok(Self::All),
            "--division" => Ok(Self::Division),
            "--effort" => Ok(Self::Effort),
            "--due" => Ok(Self::Due),
            "--on" => Ok(Self::On),
            "--flags" => Ok(Self::Flags),
            "--worktree" => Ok(Self::Worktree),
            "--dry-run" => Ok(Self::DryRun),
            "--severity" => Ok(Self::Severity),
            "--scenario" => Ok(Self::Scenario),
            "--actor" => Ok(Self::Actor),
            "--scratch" => Ok(Self::Scratch),
            "--instruments" => Ok(Self::Instruments),
            "--serves" => Ok(Self::Serves),
            "--caused-by" => Ok(Self::CausedBy),
            "--project" => Ok(Self::Project),
            "--root" => Ok(Self::Root),
            "--row" => Ok(Self::Row),
            "--span" => Ok(Self::Span),
            "--gate" => Ok(Self::Gate),
            other => Err(CliFailure::raised::<Misuse>(format!(
                "unknown option {other}; put text starting with -- after a bare --"
            ))),
        }
    }
}

impl fmt::Display for OptionName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Source => "--source",
            Self::Tag => "--tag",
            Self::Addresses => "--addresses",
            Self::From => "--from",
            Self::All => "--all",
            Self::Division => "--division",
            Self::Effort => "--effort",
            Self::Due => "--due",
            Self::On => "--on",
            Self::Flags => "--flags",
            Self::Worktree => "--worktree",
            Self::DryRun => "--dry-run",
            Self::Severity => "--severity",
            Self::Scenario => "--scenario",
            Self::Actor => "--actor",
            Self::Scratch => "--scratch",
            Self::Instruments => "--instruments",
            Self::Serves => "--serves",
            Self::CausedBy => "--caused-by",
            Self::Project => "--project",
            Self::Root => "--root",
            Self::Row => "--row",
            Self::Span => "--span",
            Self::Gate => "--gate",
        };
        formatter.write_str(name)
    }
}
