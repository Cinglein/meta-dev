use std::cell::RefCell;

use meta_log::{LogFailure, LogStore, MemoryLog, Snapshot};
use meta_schema::Record;

use super::{OPEN_DASHED, OWNER, PROGRAM};
use crate::request::Request;
use crate::request::cli_failure::{CliFailure, Invalid};
use crate::request::scale_policy::ScalePolicy;

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
        let args = std::iter::once(PROGRAM).chain(words.iter().copied()).map(String::from).collect::<Vec<_>>();
        let actor = || OWNER.parse().map_err(CliFailure::raised::<Invalid>);
        Request::try_from(args)
            .and_then(|request| request.answer_from(self, &ScalePolicy::default(), actor))
            .map(|_| ())
            .unwrap();
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
