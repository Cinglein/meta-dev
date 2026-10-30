mod step_outcome;

use std::str::FromStr;

use task::{CheckName, Failure};

use step_outcome::StepOutcome;

#[derive(Debug)]
pub struct StepOutcomes(Vec<StepOutcome>);

impl StepOutcomes {
    pub const AMBIENT: &str = "META_CI_STEPS";

    pub fn first_passed(&self) -> bool {
        self.0.first().and_then(StepOutcome::passing).is_some()
    }

    pub fn later_failures(&self) -> impl Iterator<Item = CheckName> + '_ {
        self.0.iter().skip(1).filter_map(StepOutcome::failing)
    }
}

impl FromStr for StepOutcomes {
    type Err = Failure;

    fn from_str(listed: &str) -> Result<StepOutcomes, Failure> {
        listed.split(',').map(str::parse).collect::<Result<Vec<StepOutcome>, Failure>>().map(StepOutcomes)
    }
}
