use std::fmt;

use super::domain_name::DomainName;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExternalClass {
    General,
    Framework,
    Domain(DomainName),
}

impl fmt::Display for ExternalClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExternalClass::General => formatter.write_str("general"),
            ExternalClass::Framework => formatter.write_str("framework"),
            ExternalClass::Domain(domain) => write!(formatter, "domain {domain}"),
        }
    }
}
