use std::ffi::OsString;

use meta_run::{Ran, Stderr};

use crate::git_failure::GitFailure;
use crate::remote_listing::RemoteListing;
use crate::remote_name::RemoteName;
use crate::remote_query::RemoteQuery;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Heads;

impl RemoteQuery for Heads {
    type View = Result<RemoteListing, Stderr>;

    fn words(&self, remote: &RemoteName) -> Vec<OsString> {
        ["ls-remote", "--heads"].map(OsString::from).into_iter().chain([OsString::from(remote.as_ref())]).collect()
    }

    fn view(&self, ran: Ran) -> Result<Result<RemoteListing, Stderr>, GitFailure> {
        if ran.success() { Ok(Ok(ran.stdout().text().parse()?)) } else { Ok(Err(ran.stderr().clone())) }
    }
}
