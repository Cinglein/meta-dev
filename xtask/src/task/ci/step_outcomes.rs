mod step_outcome;

use std::str::FromStr;

use task::{CheckName, Failure};

use step_outcome::StepOutcome;

#[derive(Debug)]
pub struct StepOutcomes(Vec<StepOutcome>);

impl StepOutcomes {
    pub const AMBIENT: &str = "META_CI_STEPS";
    const FIRST: &str = "ci";

    pub fn first_passed(&self) -> bool {
        self.0.first().and_then(StepOutcome::passing).is_some()
    }

    pub fn later_unsound(&self) -> impl Iterator<Item = CheckName> + '_ {
        self.0.iter().skip(1).filter_map(StepOutcome::unsound)
    }
}

impl FromStr for StepOutcomes {
    type Err = Failure;

    fn from_str(listed: &str) -> Result<StepOutcomes, Failure> {
        let steps = listed.split(',').map(str::parse).collect::<Result<Vec<StepOutcome>, Failure>>()?;
        if steps.first().is_some_and(|step| step.is_named(Self::FIRST)) {
            Ok(StepOutcomes(steps))
        } else {
            Err(Failure::Usage(format!("the first step must be {}", Self::FIRST)))
        }
    }
}
