use std::fmt;
use std::path::Path;

use crate::git_failure::GitFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    name: String,
    email: String,
}

impl Identity {
    const KEYS: [[&'static str; 2]; 3] =
        [["user.name", "user.email"], ["author.name", "author.email"], ["committer.name", "committer.email"]];

    pub(crate) fn pinned(&self, file: &Path) -> Result<(), GitFailure> {
        let mut config = git2::Config::open(file)?;
        Self::KEYS.iter().try_for_each(|[name, email]| {
            config.set_str(name, &self.name)?;
            config.set_str(email, &self.email)
        })?;
        Ok(())
    }

    #[must_use]
    pub fn new(name: String, email: String) -> Identity {
        Identity { name, email }
    }

    pub(crate) fn signed_now(&self) -> Result<git2::Signature<'static>, GitFailure> {
        Ok(git2::Signature::now(&self.name, &self.email)?)
    }
}

impl From<&git2::Signature<'_>> for Identity {
    fn from(signature: &git2::Signature<'_>) -> Identity {
        let text = |part: &[u8]| String::from_utf8_lossy(part).into_owned();
        Identity { name: text(signature.name_bytes()), email: text(signature.email_bytes()) }
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} <{}>", self.name, self.email)
    }
}
