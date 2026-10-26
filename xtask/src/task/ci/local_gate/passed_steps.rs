use std::str::FromStr;

use strum::IntoEnumIterator;

use crate::task::ci::CiStep;
use crate::task::ci::local_gate::Unready;
use crate::task::failure::Failure;

#[derive(Debug, PartialEq, Eq)]
pub struct PassedSteps(Vec<CiStep>);

impl PassedSteps {
    const SEPARATOR: &str = ", ";
    const SPACE: char = ' ';
    const FAILED: &str = "failed";

    pub fn whole(&self) -> bool {
        self.0.iter().copied().eq(CiStep::iter().take(self.0.len()))
    }
}

impl FromStr for PassedSteps {
    type Err = Failure;

    fn from_str(recorded: &str) -> Result<PassedSteps, Failure> {
        recorded
            .split(Self::SEPARATOR)
            .map(|entry| {
                entry
                    .split_once(Self::SPACE)
                    .filter(|(_, time)| *time != Self::FAILED)
                    .and_then(|(name, _)| name.parse().ok())
                    .ok_or(Failure::Unready(Unready::Incomplete))
            })
            .collect::<Result<Vec<CiStep>, Failure>>()
            .map(PassedSteps)
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::PassedSteps;
    use crate::task::ci::CiStep;

    const SOME: &str = "fmt 1.0s, clippy 2.0s";
    const FAILING: &str = "fmt 1.0s, clippy failed";

    #[test]
    fn only_every_step_passing_in_order_is_whole() {
        let every = CiStep::iter().map(|step| format!("{step} 1.0s")).collect::<Vec<String>>().join(", ");
        assert!(every.parse::<PassedSteps>().is_ok_and(|steps| steps.whole()));
        assert!(SOME.parse::<PassedSteps>().is_ok_and(|steps| !steps.whole()));
        assert!(FAILING.parse::<PassedSteps>().is_err() && "".parse::<PassedSteps>().is_err());
    }
}
