use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::request::local::local_clone::vetting::change::Change;
use crate::request::local::local_clone::vetting::vet_argv::VetArgv;
use crate::request::local::local_clone::vetting::vetted::Step;

#[derive(Clone, Debug)]
pub struct Stepped {
    step: Step,
    path: PathBuf,
}

impl Stepped {
    pub fn new(step: Step, path: &Path) -> Self {
        Self { step, path: path.to_path_buf() }
    }
}

impl Change for Stepped {
    fn argv(&self) -> VetArgv {
        match self.step {
            Step::Lock => VetArgv::Lock,
            Step::Unlock => VetArgv::Unlock,
            Step::Remove => VetArgv::Remove,
        }
    }

    fn operands(&self) -> Vec<OsString> {
        vec![self.path.clone().into_os_string()]
    }
}
