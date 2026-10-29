use std::fmt;
use std::path::Path;

use meta_git::{GitFailure, Reads as _, Repository, SignerEmails, Signers};
use meta_schema::Email;

use crate::request::output::Listed;

#[derive(Debug, PartialEq, Eq)]
pub struct Identity(Option<Email>);

impl Identity {
    const UNSET: &'static str = "no commit email";

    pub fn refusal_in(directory: &Path) -> Option<Listed> {
        let unresolved =
            Listed::new(format!("cannot resolve the commit directory; use an absolute path: {}", directory.display()));
        (!directory.is_dir()).then_some(unresolved).or_else(|| match Self::signers_in(directory) {
            Ok(signers) => signers.emails().find_map(Self::foreign).map(Listed::new),
            Err(failure) => Some(Listed::new(format!(
                "cannot check the commit email in {}: {failure}; retry, or ask the owner",
                directory.display()
            ))),
        })
    }

    fn signers_in(directory: &Path) -> Result<SignerEmails, GitFailure> {
        Repository::discovered(directory)?
            .map_or_else(|| Signers.outside_repositories(), |repository| repository.read(&Signers))
    }

    fn foreign(email: Option<&str>) -> Option<Identity> {
        let email = email.map(Email::from);
        Some(Identity(email)).filter(|identity| !identity.0.as_ref().is_some_and(Email::noreply))
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(email) => write!(formatter, "{email}"),
            None => formatter.write_str(Self::UNSET),
        }?;
        write!(
            formatter,
            " is not noreply; git config user.name NAME && git config user.email ID+NAME{}",
            Email::NOREPLY_DOMAIN
        )
    }
}
