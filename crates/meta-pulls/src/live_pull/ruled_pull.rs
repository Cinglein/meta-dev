use std::fmt;

use super::LivePull;
use super::checks::Checks;
use super::pull_state::PullState;
use super::rollup::Rollup;
use crate::required_checks::RequiredChecks;

#[derive(Clone, Copy, Debug)]
pub struct RuledPull<'pull> {
    pull: &'pull LivePull,
    required: Option<&'pull RequiredChecks>,
}

impl<'pull> RuledPull<'pull> {
    #[must_use]
    pub fn new(pull: &'pull LivePull, required: Option<&'pull RequiredChecks>) -> Option<Self> {
        (required.is_some() || pull.state != PullState::Open).then_some(Self { pull, required })
    }

    #[must_use]
    pub fn open(self) -> Option<()> {
        (self.pull.state == PullState::Open).then_some(())
    }

    #[must_use]
    pub fn state(self) -> PullState {
        self.pull.state
    }

    #[must_use]
    pub fn red(self) -> Option<Self> {
        self.open()?;
        self.checks(self.required?).failing().map(|_| self)
    }

    #[must_use]
    pub fn truncated(self) -> Option<Checks> {
        self.open()?;
        let checks = self.checks(self.required?);
        checks.truncated().map(|()| checks)
    }

    fn checks(self, required: &RequiredChecks) -> Checks {
        self.pull.rollup.as_ref().map_or_else(|| Rollup::default().checks(required), |rollup| rollup.checks(required))
    }
}

impl fmt::Display for RuledPull<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.pull.state)?;
        self.required.map_or(Ok(()), |required| write!(formatter, " {}", self.checks(required)))
    }
}
