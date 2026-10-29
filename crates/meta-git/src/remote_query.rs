use std::ffi::OsString;

use meta_run::Ran;

use crate::git_failure::GitFailure;
use crate::remote_name::RemoteName;

pub trait RemoteQuery {
    type View;

    fn words(&self, remote: &RemoteName) -> Vec<OsString>;

    fn view(&self, ran: Ran) -> Result<Self::View, GitFailure>;
}
