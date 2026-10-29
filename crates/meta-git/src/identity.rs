use crate::git_failure::GitFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    name: String,
    email: String,
}

impl Identity {
    #[must_use]
    pub fn new(name: String, email: String) -> Identity {
        Identity { name, email }
    }

    pub(crate) fn signed_now(&self) -> Result<git2::Signature<'static>, GitFailure> {
        Ok(git2::Signature::now(&self.name, &self.email)?)
    }
}
