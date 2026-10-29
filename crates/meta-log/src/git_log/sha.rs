mod not_commit_id;

use std::fmt;
use std::str::FromStr;

use meta_git::{GitFailure, ObjectId};
pub use not_commit_id::NotCommitId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sha(String);

impl Sha {
    const HEX_LENGTH: usize = 40;

    pub(crate) fn object_id(&self) -> Result<ObjectId, GitFailure> {
        self.0.parse()
    }
}

impl From<ObjectId> for Sha {
    fn from(id: ObjectId) -> Self {
        Self(id.to_string())
    }
}

impl FromStr for Sha {
    type Err = NotCommitId;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let id = text.trim();
        Some(id)
            .filter(|id| id.len() == Self::HEX_LENGTH && id.chars().all(|digit| digit.is_ascii_hexdigit()))
            .map(|id| Self(id.to_owned()))
            .ok_or_else(|| NotCommitId::new(id))
    }
}

impl fmt::Display for Sha {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Sha;

    const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
    const PADDED: &str = "0123456789abcdef0123456789abcdef01234567\n";
    const REFUSED: [&str; 6] = [
        "HEAD",
        "-p0123456789abcdef0123456789abcdef012345",
        "0123456789abcdef0123456789abcdef0123456",
        "0123456789abcdef0123456789abcdef012345678",
        "g123456789abcdef0123456789abcdef01234567",
        "",
    ];

    #[test]
    fn only_a_forty_hex_commit_id_parses() {
        assert!(REFUSED.iter().all(|text| text.parse::<Sha>().is_err()));
        assert_eq!(PADDED.parse::<Sha>().map(|sha| sha.to_string()).ok().as_deref(), Some(COMMIT));
    }
}
