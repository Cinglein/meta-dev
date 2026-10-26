mod author_name;
mod noreply_email;

use serde::Deserialize;

use author_name::AuthorName;
use noreply_email::NoreplyEmail;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    name: AuthorName,
    email: NoreplyEmail,
}

impl Identity {
    pub fn name(&self) -> &AuthorName {
        &self.name
    }

    pub fn email(&self) -> &NoreplyEmail {
        &self.email
    }
}
