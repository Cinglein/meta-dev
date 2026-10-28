use std::fmt;
use std::io::ErrorKind;

use crate::command_line::CommandLine;
use crate::stderr::Stderr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unread {
    Failed(Stderr),
    Unspawned(CommandLine, ErrorKind),
    Late(CommandLine),
}

impl fmt::Display for Unread {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed(stderr) => formatter.write_str(stderr.first_line().unwrap_or_default()),
            Self::Unspawned(line, kind) => write!(formatter, "{line}: {kind}"),
            Self::Late(line) => write!(formatter, "`{line}` ran out of time"),
        }
    }
}
