use std::fs::File;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::SpanKind::{Command, Fs};
use crate::Verb::{CiWait, Hook, Status};
use crate::trace_id::TraceId;
use crate::{Micros, Retention, SpanId, SpanRecord, Stitch, Trace, TraceDir, Verb};

const NAMES: [&str; 3] = ["0000000000000001-1", "0000000000000001-2", "0000000000000002-1"];
const VERBS: [Option<Verb>; 3] = [Some(Hook), None, Some(Status)];
const AGES: [Duration; 3] = [Duration::from_secs(1), Duration::from_secs(3), Duration::from_secs(2)];
const OLDEST: &str = "0000000000000000";
const RETAINED: [&str; 2] = [r#"{"traces": 2}"#, r#"{"traces": 1000000, "bytes": 0}"#];
const CADENCE: &str = r#"{"traces": 20}"#;
const PRUNED_AT: [Micros; 2] = [Micros::new(4), Micros::new(3)];
const FILES_LEN: usize = 3;
const CORRUPT: &str = "corrupt trace: span 100000001 recorded twice\n";
const DROPPED: &str = "dropped";
const FUTURE: &str = "ffffffffffffffff-100000001";

struct Written;

impl Written {
    const EXTENSION: &'static str = "mtr";

    fn trace(verb: Option<Verb>) -> Trace {
        let span = verb.map(|verb| SpanRecord::new(SpanId::new(1, 0), None, Command(verb), Micros::default()));
        Trace::new(span.into_iter().collect())
    }

    fn file(dir: &Path, name: &str, verb: Option<Verb>, age: Duration) {
        let path = dir.join(name).with_extension(Self::EXTENSION);
        std::fs::write(&path, Vec::from(&Self::trace(verb))).unwrap();
        File::options().write(true).open(&path).unwrap().set_modified(SystemTime::UNIX_EPOCH + age).unwrap();
    }

    fn retention(retained: &str) -> Retention {
        serde_json::from_str(retained).unwrap()
    }
}

#[test]
fn the_newest_written_trace_is_latest_and_retention_keeps_or_prunes_whole_traces() {
    let dir = tempfile::tempdir().unwrap();
    let traces = TraceDir::new(dir.path().to_path_buf());
    NAMES.iter().zip(VERBS).zip(AGES).for_each(|((name, verb), age)| Written::file(dir.path(), name, verb, age));
    let latest = |verb| traces.latest(None).unwrap().is_some_and(|trace| trace.started_by(Some(verb)));
    let before = latest(Hook);
    traces.kept(OLDEST.parse().unwrap(), &Written::trace(Some(CiWait)), Written::retention(RETAINED[0])).unwrap();
    let left = std::fs::read_dir(dir.path()).unwrap().count();
    assert_eq!((before, latest(CiWait), left, traces.latest(Some(Status)).unwrap()), (true, true, FILES_LEN, None));
}

#[test]
fn a_write_that_fails_still_prunes_older_traces_and_a_write_that_succeeds_prunes_once_per_root_of_the_count() {
    let dir = tempfile::tempdir().unwrap();
    let traces = TraceDir::new(dir.path().to_path_buf());
    Written::file(dir.path(), NAMES[0], VERBS[0], AGES[0]);
    let blocked: TraceId = OLDEST.parse().unwrap();
    std::fs::create_dir(blocked.file_in(dir.path(), Written::EXTENSION)).unwrap();
    let kept = traces.kept(blocked, &Written::trace(VERBS[0]), Written::retention(RETAINED[1]));
    assert!(kept.is_err() && traces.latest(None).unwrap().is_none());
    assert_eq!(PRUNED_AT.map(|at| Written::retention(CADENCE).due(at)), [true, false]);
}

#[test]
fn a_repeated_span_shows_as_corrupt_and_a_parent_from_the_future_is_refused() {
    let id = SpanId::new(1, 0);
    let looped = [
        SpanRecord::new(id, None, Command(Status), Micros::default()),
        SpanRecord::new(id, Some(id), Fs, Micros::default()),
    ];
    let trace = Trace::new(looped.to_vec());
    assert!(trace.to_string().contains(CORRUPT) && !trace.to_string().contains(DROPPED), "{trace}");
    assert!(FUTURE.parse::<Stitch>().is_err());
}
