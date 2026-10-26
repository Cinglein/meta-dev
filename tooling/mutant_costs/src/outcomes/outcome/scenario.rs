use serde::Deserialize;

use task::CrateName;

#[derive(Debug, Deserialize)]
pub enum Scenario {
    Baseline,
    Mutant { package: CrateName },
}

impl Scenario {
    #[must_use]
    pub fn package(&self) -> Option<&CrateName> {
        match self {
            Scenario::Mutant { package } => Some(package),
            Scenario::Baseline => None,
        }
    }
}
