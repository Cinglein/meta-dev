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
    type View = String;

    fn answered(&self, repository: &git2::Repository) -> Result<String, GitFailure> {
        let remote = repository.find_remote(&self.0.as_ref().to_string_lossy())?;
        let url = remote.url_bytes();
        String::from_utf8(url.to_vec()).map_err(|_| GitFailure::NotText(String::from_utf8_lossy(url).into_owned()))
    }
}
