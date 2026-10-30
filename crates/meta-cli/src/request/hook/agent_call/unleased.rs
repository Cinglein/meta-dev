use std::fmt;
use std::path::PathBuf;

use crate::request::output::Listed;

#[derive(Debug)]
pub enum Unleased {
    NoPrompt,
    NoProject(PathBuf),
    Refused(Listed),
}

impl Unleased {
    const HOW: &'static str =
        "start an agent with `meta claim <task> --agent <name>` and pass its output as the prompt";
}

impl fmt::Display for Unleased {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}; ", Self::HOW)?;
        match self {
            Self::NoPrompt => formatter.write_str("no prompt"),
            Self::NoProject(directory) => write!(formatter, "no meta project at {}", directory.display()),
            Self::Refused(reason) => write!(formatter, "{reason}"),
        }
    }
}
