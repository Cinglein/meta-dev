use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::query::Query;
use crate::request::local::local_clone::vetting::vet_argv::VetArgv;
use crate::request::local::local_clone::vetting::vetted::Porcelain;

#[derive(Clone, Copy, Debug)]
pub struct StatusEntries;

impl Query for StatusEntries {
    type View = Porcelain;
    const ARGV: VetArgv = VetArgv::Status;

    fn viewed(&self, out: &str) -> Result<Porcelain, Kept> {
        let Ok(status) = out.parse();
        Ok(status)
    }
}
