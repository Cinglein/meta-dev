mod detail;
mod git_log;
mod log_failure;
mod skipped_line;
mod snapshot;

pub use git_log::GitLog;
pub use log_failure::LogFailure;

use meta_schema::Record;
use snapshot::Snapshot;

pub trait LogStore {
    fn read(&self) -> Result<Snapshot, LogFailure>;
    fn append(&self, decide: &mut dyn FnMut(&Snapshot) -> Result<Record, LogFailure>) -> Result<Record, LogFailure>;
}
