use crate::change::Change;
use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::ref_name::RefName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefMove {
    name: RefName,
    from: Option<ObjectId>,
    to: ObjectId,
}

impl RefMove {
    const REASON: &'static str = "meta";

    #[must_use]
    pub fn new(name: RefName, from: Option<ObjectId>, to: ObjectId) -> RefMove {
        RefMove { name, from, to }
    }

    fn refused(&self, error: git2::Error) -> GitFailure {
        match error.code() {
            git2::ErrorCode::Modified | git2::ErrorCode::Exists | git2::ErrorCode::NotFound => {
                GitFailure::RefMoved(self.name.clone())
            }
            _ => GitFailure::Library(error),
        }
    }
}

impl Change for RefMove {
    type Applied = ();

    fn applied(&self, repository: &git2::Repository) -> Result<(), GitFailure> {
        let (name, to) = (self.name.as_str(), self.to.oid());
        let moved = match self.from {
            Some(from) => repository.reference_matching(name, to, true, from.oid(), Self::REASON),
            None => repository.reference(name, to, false, Self::REASON),
        };
        moved.map(drop).map_err(|error| self.refused(error))
    }
}
