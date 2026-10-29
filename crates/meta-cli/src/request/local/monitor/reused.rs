use thiserror::Error;

use super::process_row::Pid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
#[error("pid {0} no longer runs the process first seen (start time changed); not signalled")]
pub struct Reused(pub Pid);
