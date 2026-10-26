use std::cell::RefCell;

use meta_schema::Record;

use crate::LogStore;
use crate::log_failure::LogFailure;
use crate::snapshot::Snapshot;

#[derive(Debug, Default)]
pub struct MemoryLog {
    records: RefCell<Vec<Record>>,
}

impl LogStore for MemoryLog {
    fn read(&self) -> Result<Snapshot, LogFailure> {
        Ok(self.records.borrow().iter().cloned().fold(Snapshot::default(), Snapshot::with_record))
    }

    fn append(&self, decide: &mut dyn FnMut(&Snapshot) -> Result<Record, LogFailure>) -> Result<Record, LogFailure> {
        let record = decide(&self.read()?)?;
        self.records.borrow_mut().push(record.clone());
        Ok(record)
    }
}
