use std::fmt;

use meta_log::RemoteName;

use crate::request::project::BranchName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteRef<'name> {
    remote: &'name RemoteName,
    branch: Option<&'name BranchName>,
}

impl<'name> RemoteRef<'name> {
    const HEAD: &'static str = "HEAD";

    pub fn new(remote: &'name RemoteName, branch: Option<&'name BranchName>) -> Self {
        Self { remote, branch }
    }

    pub fn named(self) -> Option<BranchName> {
        self.to_string().parse().ok()
    }
}

impl fmt::Display for RemoteRef<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.branch {
            Some(branch) => write!(formatter, "{}/{branch}", self.remote),
            None => write!(formatter, "{}/{}", self.remote, Self::HEAD),
        }
    }
}
