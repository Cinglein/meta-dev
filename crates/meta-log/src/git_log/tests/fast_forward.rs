use std::sync::{Arc, Barrier};

use super::super::GitLog;
use super::super::content::Content;
use super::super::sha::Sha;
use super::{FIRST, Fixture, PAIR, SECOND, SINGLE};
use crate::LogStore;
use crate::log_failure::LogFailure;

impl Fixture {
    fn raced(log: &GitLog, start: &Barrier) -> Option<Sha> {
        let parent = log.fetched_tip().unwrap();
        let record = Fixture::record();
        let sibling = log.commit(parent.as_ref(), &Content::default().appended(&record).unwrap(), &record).unwrap();
        start.wait();
        log.remote.push(&sibling).unwrap().ok().map(|()| sibling)
    }
}

#[test]
fn two_writers_racing_from_one_tip_through_the_local_transport_land_exactly_one_and_refuse_the_other() {
    let fixture = Arc::new(Fixture::new(PAIR));
    fixture.log(FIRST).append::<LogFailure>(&mut |_| Ok(Fixture::record())).unwrap();
    let start = Arc::new(Barrier::new(PAIR));
    let racers = [FIRST, SECOND].map(|clone| {
        let fixture = Arc::clone(&fixture);
        let start = Arc::clone(&start);
        std::thread::spawn(move || Fixture::raced(&fixture.log(clone), &start))
    });
    let landed: Vec<_> = racers.into_iter().filter_map(|racer| racer.join().unwrap()).collect();
    assert_eq!(
        landed.iter().collect::<Vec<_>>(),
        fixture.log(SECOND).fetched_tip().unwrap().iter().collect::<Vec<_>>()
    );
}

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

#[test]
fn a_read_lists_the_remote_again_and_sees_what_another_clone_wrote() {
    let fixture = Fixture::new(PAIR);
    let reader = fixture.log(FIRST);
    let before = reader.read().unwrap().records().len();
    let written = fixture.log(SECOND).append::<LogFailure>(&mut |_| Ok(Fixture::record())).unwrap();
    let records = reader.read().unwrap().records().to_vec();
    assert!(before == 0 && records == [written]);
}
