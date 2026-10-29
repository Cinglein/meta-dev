mod author_name;
mod identity_refusal;
mod noreply_email;

use serde::Deserialize;

use author_name::AuthorName;
use identity_refusal::IdentityRefusal;
use noreply_email::NoreplyEmail;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    name: AuthorName,
    email: NoreplyEmail,
}

impl Identity {
    pub fn from_git(name: Option<&str>, email: Option<&str>) -> Result<Self, IdentityRefusal> {
        let (name, email) = name.zip(email).ok_or(IdentityRefusal::Missing)?;
        Ok(Self {
            name: name.parse().map_err(IdentityRefusal::Name)?,
            email: email.parse().map_err(IdentityRefusal::Email)?,
        })
    }

    pub fn name(&self) -> &AuthorName {
        &self.name
    }

    pub fn email(&self) -> &NoreplyEmail {
        &self.email
    }
}
