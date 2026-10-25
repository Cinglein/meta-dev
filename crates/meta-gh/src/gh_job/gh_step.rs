mod step_name;

use std::fmt;

use serde::Deserialize;

use crate::conclusion::Conclusion;
use crate::outcome::Outcome;
use step_name::StepName;

#[derive(Clone, Debug, Deserialize)]
pub struct GhStep {
    name: StepName,
    #[serde(default)]
    conclusion: Option<Conclusion>,
}

impl GhStep {
    pub fn failure(&self) -> Option<&Self> {
        self.conclusion.map(Conclusion::outcome).filter(|outcome| *outcome == Outcome::Failing).map(|_| self)
    }
}

impl fmt::Display for GhStep {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
