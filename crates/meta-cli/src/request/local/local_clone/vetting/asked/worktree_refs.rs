use std::ffi::OsString;

use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::query::Query;
use crate::request::local::local_clone::vetting::vet_argv::VetArgv;
use crate::request::local::local_clone::vetting::vetted::RefList;

#[derive(Clone, Debug)]
pub struct WorktreeRefs(String);

impl WorktreeRefs {
    pub fn new(branch: &str) -> Self {
        Self(branch.to_owned())
    }
}

impl Query for WorktreeRefs {
    type View = RefList;
    const ARGV: VetArgv = VetArgv::WorktreeRefs;

    fn operands(&self) -> Vec<OsString> {
        vec![OsString::from(&self.0)]
    }

    fn viewed(&self, out: &str) -> Result<RefList, Kept> {
        out.parse()
    }
}
