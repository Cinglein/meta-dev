use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Email(String);

impl Email {
    pub const NOREPLY_DOMAIN: &'static str = "@users.noreply.github.com";
    const WEB_COMMITTER: &'static str = "noreply@github.com";

    #[must_use]
    pub fn noreply(&self) -> bool {
        self.0.ends_with(Self::NOREPLY_DOMAIN) || self.0 == Self::WEB_COMMITTER
    }
}

impl From<&str> for Email {
    fn from(address: &str) -> Self {
        Self(address.to_owned())
    }
}

impl fmt::Display for Email {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
