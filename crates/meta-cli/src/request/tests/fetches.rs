use std::cell::RefCell;

use meta_log::{LogFailure, LogStore, MemoryLog, Snapshot};
use meta_schema::Record;
use meta_severity::Scales;

use super::{Meta, OPEN_DASHED, OWNER};

#[derive(Debug, PartialEq, Eq)]
enum Call {
    Read,
    Append,
}

#[derive(Default)]
struct Counted {
    inner: MemoryLog,
    calls: RefCell<Vec<Call>>,
}

impl LogStore for Counted {
    fn read(&self) -> Result<Snapshot, LogFailure> {
        self.calls.borrow_mut().push(Call::Read);
        self.inner.read()
    }

    fn append<Failure: From<LogFailure>>(
        &self,
        decide: &mut dyn FnMut(&Snapshot) -> Result<Record, Failure>,
    ) -> Result<Record, Failure> {
        self.calls.borrow_mut().push(Call::Append);
        self.inner.append(decide)
    }
}

impl Counted {
    fn write(&self, words: &[&str]) {
        Meta::answer(self, OWNER, &Scales::default(), words).unwrap();
    }
}

#[test]
fn a_write_to_a_marked_log_fetches_once_and_the_first_write_adds_only_the_marker() {
    let log = Counted::default();
    log.write(&OPEN_DASHED);
    assert_eq!(log.calls.take(), [Call::Append, Call::Append, Call::Append]);
    log.write(&OPEN_DASHED);
    assert_eq!(log.calls.take(), [Call::Append]);
}
