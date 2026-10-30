mod git_log;
mod log_failure;
mod memory_log;
mod skipped_line;
mod snapshot;

pub use git_log::{GitLog, NotCommitId, Remote, RemoteHeads, Sha};
pub use log_failure::LogFailure;
pub use memory_log::MemoryLog;
pub use meta_git::RemoteName;
pub use snapshot::Snapshot;

use meta_schema::Record;

pub trait LogStore {
    fn read(&self) -> Result<Snapshot, LogFailure>;
    fn append<Failure: From<LogFailure>>(
        &self,
        decide: &mut dyn FnMut(&Snapshot) -> Result<Record, Failure>,
    ) -> Result<Record, Failure>;
}
