use std::path::Path;
use std::str::FromStr;
use std::time::Instant;

use meta_run::{Run, RunFailure};
use meta_schema::Email;

use crate::request::output::Listed;

#[derive(Debug, PartialEq, Eq)]
pub struct Identity(Email);

impl Identity {
    const IDENTS: [&'static str; 2] = ["GIT_AUTHOR_IDENT", "GIT_COMMITTER_IDENT"];

    pub fn refusal_in(directory: &Path, deadline: Instant) -> Option<Listed> {
        let unresolved =
            Listed::new(format!("cannot resolve the commit directory; use an absolute path: {}", directory.display()));
        (!directory.is_dir()).then_some(unresolved).or_else(|| match Self::foreign_in(directory, deadline) {
            Ok(foreign) => foreign?.refusal(),
            Err(failure) => {
                Some(Listed::new(format!("cannot check the commit email: {failure}; retry, or ask the owner")))
            }
        })
    }

    fn foreign_in(directory: &Path, deadline: Instant) -> Result<Option<Identity>, RunFailure> {
        let shown = Self::IDENTS.iter().map(|ident| Run::git(directory).args(["var", ident]).ran(deadline));
        let foreign =
            shown.into_iter().find_map(|shown| match shown.map(|shown| shown.stdout().text().parse::<Identity>()) {
                Ok(Ok(identity)) => (!identity.0.noreply()).then_some(Ok(identity)),
                Err(failure) => Some(Err(failure)),
            });
        foreign.transpose()
    }

    fn refusal(&self) -> Option<Listed> {
        (!self.0.noreply()).then(|| {
            Listed::new(format!(
                "{} is not noreply; git config user.name NAME && git config user.email ID+NAME{}",
                self.0,
                Email::NOREPLY_DOMAIN
            ))
        })
    }
}

impl FromStr for Identity {
    type Err = std::convert::Infallible;

    fn from_str(ident: &str) -> Result<Self, Self::Err> {
        let email = ident.split_once('<').and_then(|(_, rest)| rest.split_once('>')).map_or("", |(email, _)| email);
        Ok(Self(Email::from(email)))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::Identity;

    const UNDECIDED: &str = "cannot check the commit email";
    const FOREIGN: &str = "Some One <someone@example.com> 1700000000 +0000\n";
    const NOREPLY: &str = "Some One <1+someone@users.noreply.github.com> 1700000000 +0000\n";
    const NAMED: &str = "someone@example.com is not noreply; git config user.name";

    #[test]
    fn only_a_github_noreply_email_passes_and_a_check_out_of_time_refuses() {
        let refusal =
            |ident: &str| ident.parse::<Identity>().map(|identity| identity.refusal().map(|why| why.to_string()));
        assert!(refusal(FOREIGN).is_ok_and(|why| why.is_some_and(|why| why.starts_with(NAMED))));
        assert!(refusal(NOREPLY).is_ok_and(|why| why.is_none()));
        let late = Identity::refusal_in(&std::env::temp_dir(), Instant::now()).map(|why| why.to_string());
        assert!(late.is_some_and(|why| why.starts_with(UNDECIDED)));
    }
}
