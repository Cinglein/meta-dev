use std::fmt;

use task::{CrateCategory, Dependency, ExternalClass};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dependent {
    Crate(CrateCategory),
    Tests(CrateCategory),
}

impl Dependent {
    pub fn of(category: CrateCategory, dependency: &Dependency) -> Dependent {
        if dependency.for_tests() {
            Dependent::Tests(category)
        } else {
            Dependent::Crate(category)
        }
    }

    pub fn may_depend_on(self, target: CrateCategory) -> bool {
        self.categories()
            .iter()
            .any(|category| category.may_depend_on(target))
    }

    pub fn may_use(self, class: &ExternalClass) -> bool {
        self.categories()
            .iter()
            .any(|category| category.may_use(class))
    }

    fn categories(self) -> [CrateCategory; 2] {
        match self {
            Dependent::Crate(category) => [category, category],
            Dependent::Tests(category) => [category, CrateCategory::TestSupport],
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
