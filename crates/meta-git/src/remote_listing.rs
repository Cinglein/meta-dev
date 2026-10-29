use std::collections::BTreeMap;
use std::str::FromStr;

use crate::git_failure::GitFailure;
use crate::object_id::ObjectId;
use crate::ref_name::RefName;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RemoteListing(BTreeMap<RefName, ObjectId>);

impl RemoteListing {
    const SEPARATOR: char = '\t';

    #[must_use]
    pub fn target(&self, name: &RefName) -> Option<ObjectId> {
        self.0.get(name).copied()
    }
}

impl FromStr for RemoteListing {
    type Err = GitFailure;

    fn from_str(listed: &str) -> Result<RemoteListing, GitFailure> {
        listed
            .lines()
            .map(|line| {
                let (id, name) =
                    line.split_once(Self::SEPARATOR).ok_or_else(|| GitFailure::NotObjectId(line.to_owned()))?;
                let Ok(name) = name.parse();
                Ok((name, id.parse()?))
            })
            .collect::<Result<_, GitFailure>>()
            .map(RemoteListing)
    }
}
