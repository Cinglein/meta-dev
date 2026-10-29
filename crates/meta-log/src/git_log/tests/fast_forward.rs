use super::super::content::Content;
use super::{FIRST, Fixture, SINGLE};
use crate::LogStore;
use crate::log_failure::LogFailure;

#[test]
fn a_push_that_does_not_fast_forward_the_remote_log_is_refused() {
    let fixture = Fixture::new(SINGLE);
    let log = fixture.log(FIRST);
    let record = log.append::<LogFailure>(&mut |_| Ok(Fixture::record())).unwrap();
    let parent = log.fetched_tip().unwrap();
    log.append::<LogFailure>(&mut |_| Ok(Fixture::record())).unwrap();
    let tip = log.fetched_tip().unwrap();
    let sibling = log.commit(parent.as_ref(), &Content::default().appended(&record).unwrap(), &record).unwrap();
    let pushed = log.remote.push(&sibling).unwrap();
    assert!(pushed.is_err() && log.fetched_tip().unwrap() == tip);
}
