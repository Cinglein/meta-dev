use std::fmt;
use std::path::PathBuf;

use crate::request::output::Listed;

#[derive(Debug)]
pub enum Unleased {
    Nested,
    NoPrompt,
    NoProject(PathBuf),
    Relative(PathBuf),
    Elsewhere(PathBuf),
    Unregistered(PathBuf, Listed),
    Unknown(PathBuf),
    Refused(Listed),
}

impl Unleased {
    const HOW: &'static str = "start an agent from the main session with `meta claim <task> --agent <name>` and pass its output as the prompt";
    const REGISTER: &'static str = "list each meta clone root as roots = [\"<path>\"] in";
}

impl fmt::Display for Unleased {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}; ", Self::HOW)?;
        match self {
            Self::Nested => formatter.write_str("an agent may not start agents"),
            Self::NoPrompt => formatter.write_str("no prompt"),
            Self::NoProject(directory) => write!(formatter, "no meta project at {}", directory.display()),
            Self::Relative(named) => write!(formatter, "project: {} is not an absolute path", named.display()),
            Self::Elsewhere(named) => {
                write!(formatter, "project: {} is not this session's project", named.display())
            }
            Self::Unregistered(file, reason) => {
                write!(formatter, "{reason}; {} {}", Self::REGISTER, file.display())
            }
            Self::Unknown(named) => write!(formatter, "project: {} is not a known meta project", named.display()),
            Self::Refused(reason) => write!(formatter, "{reason}"),
        }
    }
}
