use std::ffi::OsString;

use crate::remote_change::RemoteChange;
use crate::remote_name::RemoteName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fetch(String);

impl Fetch {
    const NOT_TEXT: &'static str = "the remote's path is not UTF-8 text";
}

impl From<String> for Fetch {
    fn from(refspec: String) -> Fetch {
        Fetch(refspec)
    }
}

impl RemoteChange for Fetch {
    const HELPER: &'static str = "uploadpack";

    fn words(&self, remote: &RemoteName) -> Vec<OsString> {
        ["fetch", "--quiet", "--prune", "--no-write-fetch-head", "--no-auto-maintenance"]
            .map(OsString::from)
            .into_iter()
            .chain([OsString::from(remote.as_ref()), OsString::from(&self.0)])
            .collect()
    }

    fn moved(&self, here: &git2::Repository, there: &git2::Repository) -> Result<(), git2::Error> {
        let url = there.path().to_str().ok_or_else(|| git2::Error::from_str(Self::NOT_TEXT))?;
        let mut options = git2::FetchOptions::new();
        options.prune(git2::FetchPrune::On).update_fetchhead(false).download_tags(git2::AutotagOption::None);
        here.remote_anonymous(url)?.fetch(&[self.0.as_str()], Some(&mut options), None)
    }
}
