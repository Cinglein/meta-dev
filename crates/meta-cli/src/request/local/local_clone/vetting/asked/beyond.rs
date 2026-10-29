use std::ffi::{OsStr, OsString};

use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::query::Query;
use crate::request::local::local_clone::vetting::vet_argv::VetArgv;

#[derive(Clone, Debug)]
pub struct Beyond(Vec<OsString>);

impl Beyond {
    const NOT: &'static str = "--not";

    pub fn new<'object>(
        local: impl Iterator<Item = &'object OsStr>,
        remote: impl Iterator<Item = &'object OsStr>,
    ) -> Self {
        let words = local.chain([OsStr::new(Self::NOT)]).chain(remote).map(OsStr::to_os_string);
        Self(words.collect())
    }
}

impl Query for Beyond {
    type View = ();
    const ARGV: VetArgv = VetArgv::Beyond;

    fn operands(&self) -> Vec<OsString> {
        self.0.clone()
    }

    fn viewed(&self, out: &str) -> Result<(), Kept> {
        out.lines().next().is_none().then_some(()).ok_or(Kept::Unpushed)
    }
}
