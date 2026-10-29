use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::query::Query;
use crate::request::local::local_clone::vetting::vet_argv::VetArgv;
use crate::request::local::local_clone::vetting::vetted::Index;

#[derive(Clone, Copy, Debug)]
pub struct IndexEntries;

impl Query for IndexEntries {
    type View = Index;
    const ARGV: VetArgv = VetArgv::Index;

    fn viewed(&self, out: &str) -> Result<Index, Kept> {
        out.parse()
    }
}
