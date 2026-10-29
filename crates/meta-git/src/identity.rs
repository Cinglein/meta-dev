use std::path::Path;

use crate::git_failure::GitFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    name: String,
    email: String,
}

impl Identity {
    const SECTIONS: [&'static str; 3] = ["user", "author", "committer"];

    pub(crate) fn pinned(&self, file: &Path) -> Result<(), GitFailure> {
        let mut config = git2::Config::open(file)?;
        Self::SECTIONS.iter().try_for_each(|section| {
            config.set_str(&format!("{section}.name"), &self.name)?;
            config.set_str(&format!("{section}.email"), &self.email)
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
