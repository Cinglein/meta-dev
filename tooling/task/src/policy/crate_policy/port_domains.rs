use std::collections::{BTreeMap, BTreeSet};

use super::CratePolicy;
use super::crate_category::CrateCategory;
use super::domain_name::DomainName;
use super::misdependency::Misdependency;
use crate::crate_name::CrateName;
use crate::package_graph::{Dependency, PackageGraph};

#[derive(Debug)]
pub struct PortDomains(BTreeMap<CrateName, BTreeSet<DomainName>>);

impl PortDomains {
    pub fn of_ports(packages: &PackageGraph, policy: &CratePolicy) -> PortDomains {
        PortDomains(
            packages
                .declared()
                .iter()
                .filter(|(name, _)| policy.category(name) == Some(CrateCategory::Port))
                .map(|(name, dependencies)| (name.clone(), Self::domains(dependencies, policy)))
                .collect(),
        )
    }

    pub fn exclusivity(&self, name: &CrateName) -> Vec<Misdependency> {
        self.0
            .get(name)
            .map(|domains| {
                let spanning = (domains.len() > 1)
                    .then(|| Misdependency::Spanning(domains.iter().cloned().collect()));
                let sharing = domains.iter().filter_map(|domain| self.sharing(domain));
                spanning.into_iter().chain(sharing).collect()
            })
            .unwrap_or_default()
    }

    fn domains(dependencies: &[Dependency], policy: &CratePolicy) -> BTreeSet<DomainName> {
        dependencies
            .iter()
            .filter(|dependency| !dependency.for_tests())
            .filter_map(|dependency| policy.class(dependency.name()))
            .filter_map(|class| class.domain().cloned())
            .collect()
    }

    fn sharing(&self, domain: &DomainName) -> Option<Misdependency> {
        let sharers: Vec<CrateName> = self
            .0
            .iter()
            .filter(|(_, used)| used.contains(domain))
            .map(|(port, _)| port.clone())
            .collect();
        (sharers.len() > 1).then(|| Misdependency::Sharing {
            domain: domain.clone(),
            ports: sharers,
        })
    }
}
