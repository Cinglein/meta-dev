mod dependent;
mod misdependency;

use std::collections::{BTreeMap, BTreeSet};

use task::{
    CrateCategory, CrateName, CratePolicy, Dependency, DomainName, ExternalClass, PackageGraph,
    Site, Violation,
};

use dependent::Dependent;
use misdependency::Misdependency;

#[derive(Debug)]
pub struct CrateDependencies;

impl CrateDependencies {
    pub fn violations(packages: &PackageGraph, policy: &CratePolicy) -> Vec<Violation> {
        let ports = Self::port_domains(packages, policy);
        packages
            .declared()
            .iter()
            .flat_map(|(name, dependencies)| {
                let site = Site::File(
                    packages
                        .directories()
                        .get(name)
                        .cloned()
                        .unwrap_or_default(),
                );
                Self::findings(name, dependencies, policy, &ports)
                    .into_iter()
                    .map(move |finding| Violation::new(site.clone(), finding))
            })
            .collect()
    }

    fn findings(
        name: &CrateName,
        dependencies: &[Dependency],
        policy: &CratePolicy,
        ports: &BTreeMap<CrateName, BTreeSet<DomainName>>,
    ) -> BTreeSet<Misdependency> {
        let Some(category) = policy.category(name) else {
            return BTreeSet::from([Misdependency::Uncategorised]);
        };
        dependencies
            .iter()
            .filter_map(|dependency| {
                Self::crossing(Dependent::of(category, dependency), dependency, policy)
            })
            .chain(Self::exclusivity(name, ports))
            .collect()
    }

    fn crossing(
        from: Dependent,
        dependency: &Dependency,
        policy: &CratePolicy,
    ) -> Option<Misdependency> {
        let name = dependency.name().clone();
        if dependency.is_local() {
            return policy
                .category(dependency.name())
                .filter(|category| !from.may_depend_on(*category))
                .map(|category| Misdependency::Inward {
                    from,
                    category,
                    name,
                });
        }
        match policy.class(dependency.name()) {
            None => Some(Misdependency::Unclassified(name)),
            Some(class) if from.may_use(&class) => None,
            Some(class) => Some(Misdependency::Outward { from, class, name }),
        }
    }

    fn port_domains(
        packages: &PackageGraph,
        policy: &CratePolicy,
    ) -> BTreeMap<CrateName, BTreeSet<DomainName>> {
        packages
            .declared()
            .iter()
            .filter(|(name, _)| policy.category(name) == Some(CrateCategory::Port))
            .map(|(name, dependencies)| {
                let domains = dependencies
                    .iter()
                    .filter(|dependency| !dependency.for_tests())
                    .filter_map(|dependency| match policy.class(dependency.name()) {
                        Some(ExternalClass::Domain(domain)) => Some(domain),
                        _ => None,
                    })
                    .collect();
                (name.clone(), domains)
            })
            .collect()
    }

    fn exclusivity(
        name: &CrateName,
        ports: &BTreeMap<CrateName, BTreeSet<DomainName>>,
    ) -> Vec<Misdependency> {
        let Some(domains) = ports.get(name) else {
            return Vec::new();
        };
        let spanning =
            (domains.len() > 1).then(|| Misdependency::Spanning(domains.iter().cloned().collect()));
        let sharing = domains.iter().filter_map(|domain| {
            let sharers: Vec<CrateName> = ports
                .iter()
                .filter(|(_, used)| used.contains(domain))
                .map(|(port, _)| port.clone())
                .collect();
            (sharers.len() > 1).then(|| Misdependency::Sharing {
                domain: domain.clone(),
                ports: sharers,
            })
        });
        spanning.into_iter().chain(sharing).collect()
    }
}
