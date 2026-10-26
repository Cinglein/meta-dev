mod nonce;
mod process_id;

use std::ffi::OsStr;
use std::fmt;

use nonce::Nonce;
use process_id::ProcessId;

#[derive(Debug)]
pub struct StagedName {
    name: String,
    process: ProcessId,
    nonce: Nonce,
}

impl StagedName {
    pub fn for_file(name: &OsStr) -> Self {
        Self { name: name.to_string_lossy().into_owned(), process: ProcessId::current(), nonce: Nonce::random() }
    }
}

impl fmt::Display for StagedName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}.staged", self.name, self.process, self.nonce)
    }
}
