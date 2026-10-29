use std::fmt;

use crate::task::adoption::gap::Gap;

#[derive(Debug)]
pub struct Gaps(Vec<Gap>);

impl Gaps {
    pub fn new(gaps: Vec<Gap>) -> Gaps {
        Gaps(gaps)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Gaps {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("adoption fails closed: every tool meta-dev requires must run on every change")?;
        self.0.iter().try_for_each(|gap| write!(formatter, "\n- {gap}"))
    }
}
