mod crate_category;
mod dependent;
mod domain_name;
mod external_class;
mod misdependency;
mod port_domains;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::crate_name::CrateName;
use crate::package_graph::{Dependency, PackageGraph};
use crate::report::Report;
use crate::site::Site;
use crate::violation::Violation;
use crate_category::CrateCategory;
use dependent::Dependent;
use domain_name::DomainName;
use external_class::ExternalClass;
use misdependency::Misdependency;
use port_domains::PortDomains;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CratePolicy {
    #[serde(default)]
    general: Vec<CrateName>,
    #[serde(default)]
    framework: Vec<CrateName>,
    #[serde(default)]
    domains: BTreeMap<DomainName, CrateName>,
    #[serde(default)]
    categories: BTreeMap<CrateName, CrateCategory>,
}

impl CratePolicy {
    const HEADLINE: &str = "a crate depends only on what its category in crate_policy allows";

    #[must_use]
    pub fn report(&self, packages: &PackageGraph) -> Report {
        let ports = PortDomains::of_ports(packages, self);
        Report::new(
            Self::HEADLINE,
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
                    self.findings(name, dependencies, &ports)
                        .into_iter()
                        .map(move |finding| Violation::new(site.clone(), finding))
                })
                .collect(),
        )
    }

    fn category(&self, name: &CrateName) -> Option<CrateCategory> {
        self.categories.get(name).copied()
    }

    fn class(&self, name: &CrateName) -> Option<ExternalClass> {
        self.domains
            .iter()
            .find(|(_, chosen)| *chosen == name)
            .map(|(domain, _)| ExternalClass::Domain(domain.clone()))
            .or_else(|| {
                self.framework
                    .contains(name)
                    .then_some(ExternalClass::Framework)
            })
            .or_else(|| {
                self.general
                    .contains(name)
                    .then_some(ExternalClass::General)
            })
    }

    fn findings(
        &self,
        name: &CrateName,
        dependencies: &[Dependency],
        ports: &PortDomains,
    ) -> BTreeSet<Misdependency> {
        match self.category(name) {
            None => BTreeSet::from([Misdependency::Uncategorised]),
            Some(category) => dependencies
                .iter()
                .filter_map(|dependency| {
                    self.crossing(Dependent::declared_by(category, dependency), dependency)
                })
                .chain(ports.exclusivity(name))
                .collect(),
        }
    }

    fn crossing(&self, from: Dependent, dependency: &Dependency) -> Option<Misdependency> {
        let name = dependency.name().clone();
        match (dependency.local_name(), self.class(dependency.name())) {
            (Some(local), _) => self
                .category(local)
                .and_then(|category| from.inward(category, name)),
            (None, Some(class)) => from.outward(class, name),
            (None, None) => Some(Misdependency::Unclassified(name)),
        }
    }
}
