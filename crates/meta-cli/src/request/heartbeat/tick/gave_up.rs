use std::fmt;

use crate::request::project::TickLimit;

#[derive(Debug)]
pub struct GaveUp(TickLimit);

impl GaveUp {
    pub fn after(limit: TickLimit) -> Self {
        Self(limit)
    }
}

impl fmt::Display for GaveUp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "tick gave up after {}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::GaveUp;
    use crate::request::project::TickLimit;

    const LIMIT: Duration = Duration::from_mins(1);
    const SHOWN: &str = "tick gave up after 60s";

    #[test]
    fn giving_up_names_the_tick_limit() {
        assert_eq!(GaveUp::after(TickLimit::from(LIMIT)).to_string(), SHOWN);
    }
}
