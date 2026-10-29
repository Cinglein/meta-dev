use super::super::content::Content;
use super::{FIRST, Fixture, PAIR, SECOND, SINGLE};
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
    let sibling = log.commit(parent.as_ref(), &Content::default().appended(&record).unwrap(), &record).unwrap().unwrap();
    let pushed = log.remote.push(&sibling).unwrap();
    assert!(pushed.is_err() && log.fetched_tip().unwrap() == tip);
}

#[test]
fn a_read_lists_the_remote_again_and_sees_what_another_clone_wrote() {
    let fixture = Fixture::new(PAIR);
    let reader = fixture.log(FIRST);
    let before = reader.read().unwrap().records().len();
    let written = fixture.log(SECOND).append::<LogFailure>(&mut |_| Ok(Fixture::record())).unwrap();
    let records = reader.read().unwrap().records().to_vec();
    assert!(before == 0 && records == [written]);
}
