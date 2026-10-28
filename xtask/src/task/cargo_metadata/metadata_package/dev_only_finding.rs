use std::fmt;

use task::CrateName;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DevOnlyFinding {
    Dependency(CrateName),
    Feature(CrateName, String),
}

impl fmt::Display for DevOnlyFinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DevOnlyFinding::Dependency(name) => {
                write!(formatter, "{name} is test-only; use it under [dev-dependencies] or go through the door")
            }
            DevOnlyFinding::Feature(name, feature) => {
                write!(formatter, "{name} feature {feature} is test-only; enable it under [dev-dependencies]")
            }
        }
    }
}
