use std::collections::BTreeSet;
use std::fmt;

use crate::task::failure::Failure;

#[derive(Debug)]
pub struct ForeignEmails(BTreeSet<String>);

impl ForeignEmails {
    const NOREPLY_DOMAIN: &str = "@users.noreply.github.com";
    const WEB_COMMITTER: &str = "noreply@github.com";

    pub fn refused(emails: &str) -> Result<(), Failure> {
        let foreign: BTreeSet<String> = emails
            .lines()
            .filter(|email| !email.ends_with(Self::NOREPLY_DOMAIN) && *email != Self::WEB_COMMITTER)
            .map(str::to_owned)
            .collect();
        if foreign.is_empty() { Ok(()) } else { Err(Failure::Identity(ForeignEmails(foreign))) }
    }
}

impl fmt::Display for ForeignEmails {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let listed: Vec<&str> = self.0.iter().map(String::as_str).collect();
        write!(formatter, "commit emails are not noreply: {}", listed.join(", "))
    }
}
