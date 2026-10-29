use std::ffi::OsString;

use super::vet_argv::VetArgv;

pub trait Change {
    fn argv(&self) -> VetArgv;

    fn operands(&self) -> Vec<OsString>;
}
