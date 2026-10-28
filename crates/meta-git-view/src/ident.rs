use std::str::FromStr;

use meta_schema::Email;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ident(Email);

impl Ident {
    #[must_use]
    pub fn email(&self) -> &Email {
        &self.0
    }
}

impl FromStr for Ident {
    type Err = std::convert::Infallible;

    fn from_str(ident: &str) -> Result<Self, Self::Err> {
        let email = ident.split_once('<').and_then(|(_, rest)| rest.split_once('>')).map_or("", |(email, _)| email);
        Ok(Self(Email::from(email)))
    }
}
