use meta_pulls::{Detour, Stop};

use super::queue_run::QueueRun;
use crate::request::output::Listed;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Watched {
    Left(Listed),
    Detoured(Detour),
    Stopped(Stop),
}

impl Watched {
    pub fn folded(self, ran: QueueRun) -> QueueRun {
        match self {
            Self::Left(shown) => ran.landed(shown),
            Self::Detoured(detour) => ran.detoured(detour),
            Self::Stopped(stop) => ran.halted(&stop),
        }
    }
}
