use std::ffi::OsString;

use crate::remote_change::RemoteChange;
use crate::remote_name::RemoteName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fetch(String);

impl From<String> for Fetch {
    fn from(refspec: String) -> Fetch {
        Fetch(refspec)
    }
}

impl RemoteChange for Fetch {
    fn words(&self, remote: &RemoteName) -> Vec<OsString> {
        ["fetch", "--quiet", "--prune", "--no-write-fetch-head", "--no-auto-maintenance"]
            .map(OsString::from)
            .into_iter()
            .chain([OsString::from(remote.as_ref()), OsString::from(&self.0)])
            .collect()
    }
}
