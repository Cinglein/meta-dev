mod crate_category;
mod domain_name;
mod external_class;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::crate_name::CrateName;
pub use crate_category::CrateCategory;
pub use domain_name::DomainName;
pub use external_class::ExternalClass;

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
    #[must_use]
    pub fn category(&self, name: &CrateName) -> Option<CrateCategory> {
        self.categories.get(name).copied()
    }

    #[must_use]
    pub fn class(&self, name: &CrateName) -> Option<ExternalClass> {
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
}
