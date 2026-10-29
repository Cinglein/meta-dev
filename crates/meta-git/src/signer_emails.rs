#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignerEmails(Vec<Option<String>>);

impl SignerEmails {
    pub fn emails(&self) -> impl Iterator<Item = Option<&str>> {
        self.0.iter().map(Option::as_deref)
    }
}

impl FromIterator<Option<String>> for SignerEmails {
    fn from_iter<Emails: IntoIterator<Item = Option<String>>>(emails: Emails) -> SignerEmails {
        SignerEmails(emails.into_iter().collect())
    }
}
