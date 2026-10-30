use std::fmt;

use super::crate_category::CrateCategory;
use super::external_class::ExternalClass;
use super::misdependency::Misdependency;
use crate::crate_name::CrateName;
use crate::package_graph::Dependency;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dependent {
    Crate(CrateCategory),
    Tests(CrateCategory),
}

impl Dependent {
    pub fn declared_by(category: CrateCategory, dependency: &Dependency) -> Dependent {
        if dependency.for_tests() {
            Dependent::Tests(category)
        } else {
            Dependent::Crate(category)
        }
    }

    pub fn inward(self, category: CrateCategory, name: CrateName) -> Option<Misdependency> {
        (!self
            .checked_as()
            .iter()
            .any(|own| own.reaches().contains(&category)))
        .then_some(Misdependency::Inward {
            from: self,
            category,
            name,
        })
    }

    pub fn outward(self, class: ExternalClass, name: CrateName) -> Option<Misdependency> {
        (!self
            .checked_as()
            .iter()
            .any(|own| class.users().contains(own)))
        .then_some(Misdependency::Outward {
            from: self,
            class,
            name,
        })
    }

    fn checked_as(self) -> Vec<CrateCategory> {
        match self {
            Dependent::Crate(category) => vec![category],
            Dependent::Tests(category) => vec![category, CrateCategory::TestSupport],
        }
    }
}

impl fmt::Display for Dependent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dependent::Crate(category) => write!(formatter, "{category}"),
            Dependent::Tests(category) => write!(formatter, "{category} dev"),
        }
    }
}
