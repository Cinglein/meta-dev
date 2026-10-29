use std::ffi::OsString;
use std::fmt;

use crate::object_id::ObjectId;
use crate::ref_name::RefName;
use crate::remote_change::RemoteChange;
use crate::remote_name::RemoteName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FastForward {
    id: ObjectId,
    to: RefName,
}

impl FastForward {
    #[must_use]
    pub fn new(id: ObjectId, to: RefName) -> FastForward {
        FastForward { id, to }
    }
}

impl RemoteChange for FastForward {
    fn words(&self, remote: &RemoteName) -> Vec<OsString> {
        ["push", "--quiet"]
            .map(OsString::from)
            .into_iter()
            .chain([OsString::from(remote.as_ref()), OsString::from(self.to_string())])
            .collect()
    }
}

impl fmt::Display for FastForward {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.id, self.to)
    }
}
