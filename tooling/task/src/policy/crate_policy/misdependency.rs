use std::fmt;

use super::crate_category::CrateCategory;
use super::dependent::Dependent;
use super::domain_name::DomainName;
use super::external_class::ExternalClass;
use crate::crate_name::CrateName;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Misdependency {
    Uncategorised,
    Unclassified(CrateName),
    Inward {
        from: Dependent,
        category: CrateCategory,
        name: CrateName,
    },
    Outward {
        from: Dependent,
        class: ExternalClass,
        name: CrateName,
    },
    Spanning(Vec<DomainName>),
    Sharing {
        domain: DomainName,
        ports: Vec<CrateName>,
    },
}

impl fmt::Display for Misdependency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let listed = |names: Vec<String>| names.join(", ");
        match self {
            Misdependency::Uncategorised => formatter.write_str("no category in crate_policy"),
            Misdependency::Unclassified(name) => {
                write!(formatter, "{name}: no class in crate_policy")
            }
            Misdependency::Inward {
                from,
                category,
                name,
            } => write!(formatter, "{from} -> {category}: {name}"),
            Misdependency::Outward { from, class, name } => {
                write!(formatter, "{from} -> {class}: {name}")
            }
            Misdependency::Spanning(domains) => write!(
                formatter,
                "port spans domains: {}",
                listed(domains.iter().map(ToString::to_string).collect())
            ),
            Misdependency::Sharing { domain, ports } => write!(
                formatter,
                "domain {domain} shared by ports: {}",
                listed(ports.iter().map(ToString::to_string).collect())
            ),
        }
    }
}
