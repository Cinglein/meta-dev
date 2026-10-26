use std::fmt;

use strum::VariantNames;

use super::Task;
use task::Failure;

#[derive(Debug)]
pub struct Usage;

impl fmt::Display for Usage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "usage: cargo xtask <{}> [--affected]", Task::VARIANTS.join("|"))
    }
}

impl From<Usage> for Failure {
    fn from(usage: Usage) -> Failure {
        Failure::Usage(usage.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::Task;

    const SHOWN: &str = "usage: cargo xtask <affected|baseline|ci|";

    #[test]
    fn usage_lists_the_tasks() {
        assert!(Task::usage().to_string().starts_with(SHOWN));
    }
}
