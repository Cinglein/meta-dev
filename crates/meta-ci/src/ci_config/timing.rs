use std::fmt;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timing {
    name: String,
    elapsed: Duration,
}

impl Timing {
    pub fn new(name: &str, elapsed: Duration) -> Timing {
        Timing { name: name.to_owned(), elapsed }
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

impl fmt::Display for Timing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ci: {} {:.1?}", self.name, self.elapsed)
    }
}
