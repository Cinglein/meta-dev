use std::fmt;

use super::crate_category::CrateCategory;
use super::domain_name::DomainName;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExternalClass {
    General,
    Framework,
    Domain(DomainName),
}

impl ExternalClass {
    const EVERY: &[CrateCategory] = &[
        CrateCategory::Core,
        CrateCategory::Port,
        CrateCategory::Shell,
        CrateCategory::Bin,
        CrateCategory::Tooling,
        CrateCategory::TestSupport,
    ];
    const APPLICATIONS: &[CrateCategory] = &[CrateCategory::Bin, CrateCategory::Tooling];
    const PORTS: &[CrateCategory] = &[CrateCategory::Port];

    #[must_use]
    pub const fn users(&self) -> &'static [CrateCategory] {
        match self {
            ExternalClass::General => Self::EVERY,
            ExternalClass::Framework => Self::APPLICATIONS,
            ExternalClass::Domain(_) => Self::PORTS,
        }
    }

    #[must_use]
    pub fn domain(&self) -> Option<&DomainName> {
        match self {
            ExternalClass::Domain(domain) => Some(domain),
            ExternalClass::General | ExternalClass::Framework => None,
        }
    }
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
