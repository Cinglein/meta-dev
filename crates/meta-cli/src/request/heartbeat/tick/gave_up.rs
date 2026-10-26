use std::fmt;

use crate::request::output::Listed;
use crate::request::project::TickLimit;

#[derive(Debug)]
pub struct GaveUp {
    limit: TickLimit,
    reason: Listed,
}

impl GaveUp {
    pub fn after(limit: TickLimit, reason: Listed) -> Self {
        Self { limit, reason }
    }
}

impl fmt::Display for GaveUp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "tick gave up after {}: {}", self.limit, self.reason)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::GaveUp;
    use crate::request::output::Listed;
    use crate::request::project::TickLimit;

    const LIMIT: Duration = Duration::from_mins(1);
    const SHOWN: &str = "tick gave up after 60s: timed out";
    const REASON: &str = "timed out";

    #[test]
    fn giving_up_names_the_tick_limit_and_the_reason() {
        assert_eq!(GaveUp::after(TickLimit::from(LIMIT), Listed::new(REASON)).to_string(), SHOWN);
    }
}
