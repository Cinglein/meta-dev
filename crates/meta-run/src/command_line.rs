use std::ffi::OsStr;
use std::fmt;
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandLine(Vec<String>);

impl CommandLine {
    #[must_use]
    pub fn words(&self) -> &[String] {
        &self.0
    }
}

impl From<&Command> for CommandLine {
    fn from(command: &Command) -> Self {
        let words = [command.get_program()].into_iter().chain(command.get_args());
        Self(words.map(OsStr::to_string_lossy).map(String::from).collect())
    }
}

impl fmt::Display for CommandLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0.join(" "))
    }
}
