use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use meta_schema::Email;
use meta_trace::SpawnSpan;

use crate::request::output::Listed;

#[derive(Debug, PartialEq, Eq)]
pub struct Identity(Email);

impl Identity {
    const IDENTS: [&'static str; 2] = ["GIT_AUTHOR_IDENT", "GIT_COMMITTER_IDENT"];

    pub fn refusal_in(directory: &Path) -> Option<Listed> {
        let unresolved =
            Listed::new(format!("cannot resolve the commit directory; use an absolute path: {}", directory.display()));
        (!directory.is_dir()).then_some(unresolved).or_else(|| Self::foreign_in(directory)?.refusal())
    }

    fn foreign_in(directory: &Path) -> Option<Identity> {
        Self::IDENTS.iter().find_map(|ident| {
            let shown = SpawnSpan::output_of(Command::new("git").arg("-C").arg(directory).args(["var", ident])).ok()?;
            let Ok(identity) = String::from_utf8_lossy(&shown.stdout).parse::<Identity>();
            Some(identity).filter(|identity| !identity.0.noreply())
        })
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
    use super::Identity;

    const FOREIGN: &str = "Some One <someone@example.com> 1700000000 +0000\n";
    const NOREPLY: &str = "Some One <1+someone@users.noreply.github.com> 1700000000 +0000\n";
    const NAMED: &str = "someone@example.com is not noreply; git config user.name";

    #[test]
    fn only_a_github_noreply_email_passes() {
        let refusal =
            |ident: &str| ident.parse::<Identity>().map(|identity| identity.refusal().map(|why| why.to_string()));
        assert!(refusal(FOREIGN).is_ok_and(|why| why.is_some_and(|why| why.starts_with(NAMED))));
        assert!(refusal(NOREPLY).is_ok_and(|why| why.is_none()));
    }
}
