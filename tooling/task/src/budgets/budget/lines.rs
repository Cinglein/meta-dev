use std::fmt;

use super::allowance::Amount;
use super::state::State;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lines {
    warning: Amount,
    blocking: Amount,
}

impl Lines {
    #[must_use]
    pub fn new(warning: Amount, blocking: Amount) -> Lines {
        Lines { warning, blocking }
    }

    #[must_use]
    pub fn state(&self, spend: &Amount) -> State {
        if *spend > self.blocking {
            State::Over
        } else if *spend > self.warning {
            State::Warn
        } else {
            State::Ok
        }
    }
}

impl fmt::Display for Lines {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:>10} {:>10}",
            self.warning.to_string(),
            self.blocking.to_string()
        )
    }
}
