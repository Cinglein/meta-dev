use std::sync::Arc;

use tracing::{Id, Subscriber as _};

use crate::SpanKind::{Command, Fs};
use crate::lane::Lane;
use crate::recorder::Recorder;
use crate::{Micros, SpanId, SpanRecord, Trace, Verb};

const SHOWN: &str = "processes, 4096 spans\n1 spans dropped\n";
const LANE: Lane = Lane::new(2);
const TID: &str = r#""tid":2"#;

#[test]
fn a_full_recorder_counts_its_drops_keeps_the_parent_chain_and_ignores_a_foreign_exit() {
    let recorder = Arc::new(Recorder::new(None));
    let current = tracing::subscriber::with_default(Arc::clone(&recorder), || {
        let _command = Command(Verb::Status).entered();
        (1..Recorder::CAPACITY).for_each(|_| drop(Fs.entered()));
        let _dropped = Fs.entered();
        recorder.exit(&Id::from(SpanId::new(0, 0)));
        Recorder::current()
    });
    let kept: Trace = [Vec::from(&recorder.taken())].iter().map(|bytes| Trace::from(bytes.as_slice())).collect();
    assert_eq!(current, Some(SpanId::new(std::process::id(), 0)));
    assert!(kept.to_string().contains(SHOWN), "{kept}");
}

#[test]
fn each_thread_records_on_its_own_lane_and_the_lane_reaches_the_file_and_chrome() {
    let other = std::thread::spawn(Lane::current).join().unwrap();
    let span = SpanRecord::new(SpanId::new(1, 0), None, Fs, Micros::default()).on_lane(LANE);
    let read = SpanRecord::try_from(Vec::from(&span).first_chunk::<{ SpanRecord::LEN }>().unwrap());
    assert!(other != Lane::current() && read == Ok(span));
    assert!(Trace::new(vec![span]).chrome().unwrap().contains(TID));
}
