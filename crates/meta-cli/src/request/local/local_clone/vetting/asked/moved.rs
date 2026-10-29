use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::request::local::local_clone::vetting::change::Change;
use crate::request::local::local_clone::vetting::vet_argv::VetArgv;

#[derive(Clone, Debug)]
pub struct Moved {
    from: PathBuf,
    to: PathBuf,
}

impl Moved {
    pub fn new(from: &Path, to: &Path) -> Self {
        Self { from: from.to_path_buf(), to: to.to_path_buf() }
    }
}

impl Change for Moved {
    fn argv(&self) -> VetArgv {
        VetArgv::Move
    }

    fn operands(&self) -> Vec<OsString> {
        vec![self.from.clone().into_os_string(), self.to.clone().into_os_string()]
    }
}
