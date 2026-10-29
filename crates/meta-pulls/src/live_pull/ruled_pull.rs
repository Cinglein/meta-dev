use std::fmt;

use super::LivePull;
use super::checks::Checks;
use super::merge_state::MergeState;
use super::merging::Merging;
use super::pull_state::PullState;
use super::rollup::Rollup;
use crate::head::{Barred, Behind, Checking, Conflicted, Gone, Green, Head, Red, Unmergeable};
use crate::required_checks::RequiredChecks;
use crate::stop::Stop;
use crate::trunk::Trunk;
use crate::verdict::Verdict;

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

    #[must_use]
    pub fn head(self, trunk: &Trunk) -> Head {
        let pull = self.pull;
        let number = pull.number;
        match (pull.state, trunk.barred(number, &pull.base), pull.merging.as_ref()) {
            (PullState::Merged | PullState::Closed, ..) => Head::Gone(Gone::new(number, pull.state)),
            (PullState::Open, Some(stop), _) => Head::Barred(Barred::new(stop)),
            (PullState::Open, None, None) => Head::Barred(Barred::new(Stop::Incomplete(number))),
            (PullState::Open, None, Some(merging)) => self.open_head(merging),
        }
    }

    fn open_head(self, merging: &Merging) -> Head {
        let number = self.pull.number;
        let verdict = self.required.map_or(Verdict::Pending, |required| self.checks(required).verdict());
        match (merging.state(), verdict.failing(), verdict.passed()) {
            (MergeState::Dirty, ..) => Head::Conflicted(Conflicted::new(number)),
            (_, Some(_), _) => {
                Head::Red(Red::new(number, self.pull.rollup.as_ref().map(Rollup::failing).unwrap_or_default()))
            }
            (MergeState::Behind, None, _) => Head::Behind(Behind::new(number)),
            (MergeState::Clean | MergeState::HasHooks, None, Some(())) => {
                Head::Green(Green::new(number, merging.commit().clone()))
            }
            (MergeState::Blocked | MergeState::Draft | MergeState::Unstable, None, Some(())) => {
                Head::Unmergeable(Unmergeable::new(number, merging.state()))
            }
            (_, None, _) => Head::Checking(Checking::new(number)),
        }
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
