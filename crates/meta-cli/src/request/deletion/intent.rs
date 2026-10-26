mod deletable;

use std::path::Path;

use meta_schema::Event;

use super::kept::Kept;
use super::occupancy::Occupancy;
use super::vetted::Vetted;
use crate::request::cli_failure::CliFailure;
use crate::request::output::Output;
use deletable::Deletable;

#[derive(Debug)]
pub struct Intent {
    vetted: Vetted,
    logged: Output,
}

impl Intent {
    pub fn announced(
        vetted: Vetted,
        record: &mut impl FnMut(Event) -> Result<Output, CliFailure>,
    ) -> Result<Self, CliFailure> {
        Ok(Self { logged: record(Event::Removing(vetted.removal()))?, vetted })
    }

    pub fn vetted(&self) -> &Vetted {
        &self.vetted
    }

    pub fn carried_out(self, occupancy: &Occupancy, project: &Path) -> Result<Output, Kept> {
        Deletable::confirmed(&self.vetted, occupancy, project)?.delete()?;
        Ok(self.logged)
    }
}
