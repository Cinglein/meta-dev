mod remote_ref;

use std::ffi::OsStr;

use meta_log::RemoteName;

use super::claim_refusal::ClaimRefusal;
use super::holding::Holding;
use crate::request::project::BranchName;
use remote_ref::RemoteRef;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchRefs {
    remote: RemoteName,
    branch: BranchName,
    local: Option<()>,
    remote_branch: Option<BranchName>,
    remote_head: Option<BranchName>,
}

impl BranchRefs {
    pub fn gathered(remote: RemoteName, branch: BranchName, verified: impl Fn(&OsStr) -> Option<()>) -> Self {
        let found = |name: Option<BranchName>| name.filter(|name| verified(name.as_ref()).is_some());
        Self {
            local: verified(OsStr::new(&branch.qualified())),
            remote_branch: found(RemoteRef::new(&remote, Some(&branch)).named()),
            remote_head: found(RemoteRef::new(&remote, None).named()),
            remote,
            branch,
        }
    }

    pub fn branch(&self) -> &BranchName {
        &self.branch
    }

    pub fn base(&self, holding: Holding) -> Result<Option<BranchName>, ClaimRefusal> {
        match (self.local, holding) {
            (Some(()), Holding::Held) => Ok(None),
            (Some(()), Holding::Fresh) => Err(ClaimRefusal::Unheld(self.branch.clone())),
            (None, _) => self
                .remote_branch
                .clone()
                .or_else(|| self.remote_head.clone())
                .map(Some)
                .ok_or_else(|| ClaimRefusal::HeadUnset(self.remote.clone())),
        }
    }
}
