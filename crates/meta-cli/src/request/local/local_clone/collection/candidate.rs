use meta_status::Ending;

use super::tracked::Tracked;
use crate::request::local::local_clone::vetting::Vetted;

#[derive(Debug)]
pub struct Candidate<'census> {
    tracked: &'census Tracked,
    vetted: Vetted,
    ending: Ending,
}

impl<'census> Candidate<'census> {
    pub fn new(tracked: &'census Tracked, vetted: Vetted, ending: Ending) -> Self {
        Self { tracked, vetted, ending }
    }

    pub fn tracked(&self) -> &'census Tracked {
        self.tracked
    }

    pub fn vetted(&self) -> &Vetted {
        &self.vetted
    }

    pub fn ending(&self) -> Ending {
        self.ending
    }

    pub fn into_vetted(self) -> Vetted {
        self.vetted
    }
}
