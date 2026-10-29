use crate::object_id::ObjectId;
use crate::ref_name::RefName;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckedOut {
    commit: Option<ObjectId>,
    branch: Option<RefName>,
}

impl CheckedOut {
    pub(crate) fn new(commit: Option<ObjectId>, branch: Option<RefName>) -> CheckedOut {
        CheckedOut { commit, branch }
    }

    #[must_use]
    pub fn commit(&self) -> Option<ObjectId> {
        self.commit
    }

    #[must_use]
    pub fn branch(&self) -> Option<&RefName> {
        self.branch.as_ref()
    }
}
