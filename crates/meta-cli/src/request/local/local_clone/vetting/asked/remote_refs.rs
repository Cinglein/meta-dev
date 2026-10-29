use std::ffi::OsString;

use meta_log::RemoteName;

use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::query::Query;
use crate::request::local::local_clone::vetting::vet_argv::VetArgv;
use crate::request::local::local_clone::vetting::vetted::RefList;

#[derive(Clone, Debug)]
pub struct RemoteRefs(RemoteName);

impl RemoteRefs {
    pub fn new(remote: RemoteName) -> Self {
        Self(remote)
    }
}

impl Query for RemoteRefs {
    type View = RefList;
    const ARGV: VetArgv = VetArgv::RemoteRefs;

    fn operands(&self) -> Vec<OsString> {
        let remote: &std::ffi::OsStr = self.0.as_ref();
        vec![remote.to_os_string()]
    }

    fn viewed(&self, out: &str) -> Result<RefList, Kept> {
        out.parse()
    }
}
