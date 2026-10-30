use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Patience {
    waited: Duration,
    limit: Duration,
}

impl Patience {
    #[must_use]
    pub fn new(waited: Duration, limit: Duration) -> Self {
        Self { waited, limit }
    }

    #[must_use]
    pub fn spent(self) -> Option<()> {
        (self.waited > self.limit).then_some(())
    }
}
