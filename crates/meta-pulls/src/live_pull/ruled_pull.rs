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
        self.checks()?.failing().map(|_| self)
    }

    #[must_use]
    pub fn truncated(self) -> Option<Checks> {
        self.open()?;
        let checks = self.checks()?;
        checks.truncated().map(|()| checks)
    }

    fn checks(self) -> Option<Checks> {
        let rollup = self.pull.rollup.as_ref();
        self.required
            .map(|required| rollup.map_or_else(|| Rollup::default().checks(required), |rollup| rollup.checks(required)))
    }
}

impl fmt::Display for RuledPull<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.pull.state)?;
        self.checks().map_or(Ok(()), |checks| write!(formatter, " {checks}"))
    }
}
