use std::cell::Cell;

use meta_log::{LogFailure, LogStore, MemoryLog, Snapshot};
use meta_schema::Record;

use super::{CLAIM, OPEN_DASHED, OWNER, PROGRAM};
use crate::request::Request;
use crate::request::cli_failure::{CliFailure, Invalid};

#[derive(Default)]
struct Counted {
    inner: MemoryLog,
    reads: Cell<usize>,
    appends: Cell<usize>,
}

impl LogStore for Counted {
    fn read(&self) -> Result<Snapshot, LogFailure> {
        self.reads.set(self.reads.get() + 1);
        self.inner.read()
    }

    fn append<Failure: From<LogFailure>>(
        &self,
        decide: &mut dyn FnMut(&Snapshot) -> Result<Record, Failure>,
    ) -> Result<Record, Failure> {
        self.appends.set(self.appends.get() + 1);
        self.inner.append(decide)
    }
}

impl Counted {
    fn write(&self, words: &[&str]) -> String {
        let args = std::iter::once(PROGRAM).chain(words.iter().copied()).map(String::from).collect::<Vec<_>>();
        let actor = || OWNER.parse().map_err(CliFailure::raised::<Invalid>);
        Request::try_from(args).and_then(|request| request.answer_from(self, actor)).unwrap().to_string()
    }
}

#[test]
fn a_write_to_a_marked_log_fetches_once_and_the_first_write_adds_only_the_marker() {
    let log = Counted::default();
    let task = log.write(&OPEN_DASHED).split_whitespace().nth(super::ID_INDEX).unwrap().to_owned();
    let (reads, appends) = (log.reads.get(), log.appends.get());
    log.write(&[CLAIM, &task]);
    assert_eq!((reads, appends, log.reads.get(), log.appends.get()), (0, 3, 0, 4));
}
