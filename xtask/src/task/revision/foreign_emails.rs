use std::collections::BTreeSet;
use std::fmt;

use meta_schema::Email;

use crate::task::failure::Failure;

#[derive(Debug)]
pub struct ForeignEmails(BTreeSet<Email>);

impl ForeignEmails {
    pub fn refused(emails: &str) -> Result<(), Failure> {
        let foreign: BTreeSet<Email> = emails.lines().map(Email::from).filter(|email| !email.noreply()).collect();
        if foreign.is_empty() { Ok(()) } else { Err(Failure::Identity(ForeignEmails(foreign))) }
    }
}

impl fmt::Display for ForeignEmails {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let listed: Vec<String> = self.0.iter().map(Email::to_string).collect();
        write!(formatter, "commit emails are not noreply: {}", listed.join(", "))
    }
}
