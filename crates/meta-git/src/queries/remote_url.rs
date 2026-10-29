use std::ffi::OsStr;

use crate::git_failure::GitFailure;
use crate::query::Query;
use crate::remote_name::RemoteName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteUrl(RemoteName);

impl From<RemoteName> for RemoteUrl {
    fn from(remote: RemoteName) -> RemoteUrl {
        RemoteUrl(remote)
    }
}

impl Query for RemoteUrl {
    type View = Option<String>;

    fn answered(&self, repository: &git2::Repository) -> Result<Option<String>, GitFailure> {
        let name = AsRef::<OsStr>::as_ref(&self.0).to_string_lossy();
        Ok(repository.find_remote(&name)?.url().ok().map(str::to_owned))
    }
}
