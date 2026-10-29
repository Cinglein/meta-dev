use meta_git::CheckedOut;

use crate::request::ci::Commit;
use crate::request::project::BranchName;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Checkout {
    head: Option<Commit>,
    branch: Option<BranchName>,
}

impl Checkout {
    pub fn new(head: Option<Commit>, branch: Option<BranchName>) -> Self {
        Self { head, branch }
    }

    pub fn head(&self) -> Option<&Commit> {
        self.head.as_ref()
    }

    pub fn branch(&self) -> Option<&BranchName> {
        self.branch.as_ref()
    }
}

impl From<&CheckedOut> for Checkout {
    fn from(checked_out: &CheckedOut) -> Self {
        let branch = checked_out.branch().and_then(|name| BranchName::try_from(name).ok());
        Self { head: checked_out.commit().map(Commit::from), branch }
    }
}
