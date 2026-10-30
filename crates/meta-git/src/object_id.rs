use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::git_failure::GitFailure;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(git2::Oid);

impl ObjectId {
    const HEX_LENGTH: usize = 40;
    const FAN_OUT_LEN: usize = 2;

    pub(crate) fn oid(self) -> git2::Oid {
        self.0
    }

    pub(crate) fn fanned(self) -> PathBuf {
        let text = self.0.to_string();
        let (fan, rest) = text.split_at(Self::FAN_OUT_LEN);
        Path::new(fan).join(rest)
    }
}

impl From<git2::Oid> for ObjectId {
    fn from(oid: git2::Oid) -> ObjectId {
        ObjectId(oid)
    }
}

impl FromStr for ObjectId {
    type Err = GitFailure;

    fn from_str(text: &str) -> Result<ObjectId, GitFailure> {
        Some(text)
            .filter(|text| text.len() == Self::HEX_LENGTH && text.chars().all(|digit| digit.is_ascii_hexdigit()))
            .map_or_else(
                || Err(GitFailure::NotObjectId(text.to_owned())),
                |text| Ok(ObjectId(git2::Oid::from_str(text)?)),
            )
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
