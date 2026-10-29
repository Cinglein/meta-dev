use super::super::content::Content;
use super::{FIRST, Fixture, SINGLE};
use crate::LogStore;
use crate::log_failure::LogFailure;

#[test]
fn a_push_that_does_not_fast_forward_the_remote_log_is_refused_even_when_its_base_is_the_tip() {
    let fixture = Fixture::new(SINGLE);
    let log = fixture.log(FIRST);
    let record = log.append::<LogFailure>(&mut |_| Ok(Fixture::record())).unwrap();
    let tip = log.fetched_tip().unwrap();
    let unrelated = log.commit(None, &Content::default().appended(&record).unwrap(), &record).unwrap();
    let pushed = log.remote.push(&unrelated, tip.as_ref()).unwrap();
    assert!(!pushed.success() && log.fetched_tip().unwrap() == tip);
}
