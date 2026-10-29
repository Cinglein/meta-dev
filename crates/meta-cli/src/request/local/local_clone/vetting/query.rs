use std::ffi::OsString;

use super::kept::Kept;
use super::vet_argv::VetArgv;

pub trait Query {
    type View;
    const ARGV: VetArgv;

    fn operands(&self) -> Vec<OsString> {
        Vec::new()
    }

    fn input(&self) -> Option<String> {
        None
    }

    fn viewed(&self, out: &str) -> Result<Self::View, Kept>;
}
