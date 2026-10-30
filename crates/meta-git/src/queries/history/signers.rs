use crate::git_failure::GitFailure;
use crate::query::Query;
use crate::signer_emails::SignerEmails;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signers;

impl Signers {
    const ROLES: [(&'static str, &'static str); 2] =
        [("GIT_AUTHOR_EMAIL", "author.email"), ("GIT_COMMITTER_EMAIL", "committer.email")];
    const SHARED: &'static str = "user.email";
    const FALLBACK: &'static str = "EMAIL";

    pub fn outside_repositories(self) -> Result<SignerEmails, GitFailure> {
        Ok(Self::configured(&git2::Config::open_default()?.snapshot()?))
    }

    fn configured(config: &git2::Config) -> SignerEmails {
        let setting = |key: &str| config.get_string(key).ok();
        let variable = |name: &str| std::env::var(name).ok();
        Self::ROLES
            .iter()
            .map(|(role, key)| {
                variable(role)
                    .or_else(|| setting(key))
                    .or_else(|| setting(Self::SHARED))
                    .or_else(|| variable(Self::FALLBACK))
            })
            .collect()
    }
}

impl Query for Signers {
    type View = SignerEmails;

    fn answered(&self, repository: &git2::Repository) -> Result<SignerEmails, GitFailure> {
        Ok(Self::configured(&repository.config()?.snapshot()?))
    }
}
