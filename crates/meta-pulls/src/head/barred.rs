use crate::stop::Stop;
use crate::transition::Transition;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Barred {
    stop: Stop,
}

impl Barred {
    pub(crate) fn new(stop: Stop) -> Self {
        Self { stop }
    }

    #[must_use]
    pub fn stopped(self) -> Transition {
        Transition::Stop(self.stop)
    }
}
