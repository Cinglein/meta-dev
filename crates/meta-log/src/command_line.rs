use std::ffi::OsStr;
use std::fmt;
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandLine(Vec<String>);

impl From<&Command> for CommandLine {
    fn from(command: &Command) -> Self {
        Self(command.get_args().map(OsStr::to_string_lossy).map(String::from).collect())
    }
}

impl fmt::Display for CommandLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "git {}", self.0.join(" "))
    }
}
