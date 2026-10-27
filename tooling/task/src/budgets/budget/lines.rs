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
    pub fn blocking(&self) -> &Amount {
        &self.blocking
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

#[cfg(test)]
mod tests {
    use super::super::allowance::Amount;
    use super::super::state::State;
    use super::Lines;
    use crate::policy::Most;

    const WARNING: usize = 6;
    const BLOCKING: usize = 10;
    const SPENDS: [usize; 4] = [WARNING, WARNING + 1, BLOCKING, BLOCKING + 1];
    const STATES: [State; 4] = [State::Ok, State::Warn, State::Warn, State::Over];

    #[test]
    fn a_spend_exactly_at_a_line_stays_below_it() {
        let count = |count: usize| Amount::Count(Most::from(count));
        let lines = Lines::new(count(WARNING), count(BLOCKING));
        assert_eq!(SPENDS.map(|spend| lines.state(&count(spend))), STATES);
    }
}
