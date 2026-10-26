mod command_line;
mod git_log;
mod log_failure;
mod memory_log;
mod remote_name;
mod skipped_line;
mod snapshot;
mod stderr;

pub use git_log::GitLog;
pub use log_failure::LogFailure;
pub use memory_log::MemoryLog;
pub use remote_name::RemoteName;
pub use snapshot::Snapshot;

use meta_schema::Record;

pub trait LogStore {
    fn read(&self) -> Result<Snapshot, LogFailure>;
    fn append<Failure: From<LogFailure>>(
        &self,
        decide: &mut dyn FnMut(&Snapshot) -> Result<Record, Failure>,
    ) -> Result<Record, Failure>;
}
