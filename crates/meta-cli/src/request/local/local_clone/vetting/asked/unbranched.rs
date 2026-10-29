use std::ffi::OsString;

use crate::request::local::local_clone::vetting::change::Change;
use crate::request::local::local_clone::vetting::vet_argv::VetArgv;
use crate::request::local::local_clone::vetting::vetted::RefLine;

#[derive(Clone, Debug)]
pub struct Unbranched(RefLine);

impl Unbranched {
    pub fn new(branch: &RefLine) -> Self {
        Self(branch.clone())
    }
}

impl Change for Unbranched {
    fn argv(&self) -> VetArgv {
        VetArgv::Unbranch
    }

    fn operands(&self) -> Vec<OsString> {
        self.0.deletion().map(std::ffi::OsStr::to_os_string).collect()
    }
}
