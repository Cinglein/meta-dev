mod kept;
mod overflow;

use std::ffi::OsString;
use std::process::Command;
use std::time::Duration;

use crate::Program::{Gh, Git, Lsof, Meta, Other};
use crate::SpanKind::{Fs, Network, Spawn};
use crate::kind_code::KindCode;
use crate::{Micros, Recording, Retention, SpanId, SpanKind, SpanRecord, Stitch, TraceDir, Verb};

const COMMANDS: [(&str, &[&str]); 6] = [
    ("git", &["-C", "/repo", "fetch"]),
    ("/usr/bin/git", &["status"]),
    ("gh", &[]),
    ("meta", &[]),
    ("lsof", &[]),
    ("sh", &[]),
];
const KINDS: [SpanKind; 6] = [Network(Git), Spawn(Git), Network(Gh), Spawn(Meta), Spawn(Lsof), Spawn(Other)];
const CODED: [SpanKind; 3] = [Fs, SpanKind::Command(Verb::Other), Network(Git)];
const STITCH: &str = "00000000000000ab-500000001";
const TRACES: [&str; 4] = ["0000000000000001", "0000000000000002", "0000000000000003", "0000000000000004"];
const VERBS: [Verb; 4] = [Verb::Status, Verb::Hook, Verb::Gc, Verb::Status];
const LAST: usize = 3;
const RETAINED: [&str; 2] = [r#"{"traces": 2}"#, r#"{"traces": 3, "bytes": 96}"#];
const PAUSE: Duration = Duration::from_millis(1);
const UNTRACED: &str = "ms spawn meta\n";
const WALKED: &str = "ms fs\n";
const SECTIONS: &str = "untraced";

#[test]
fn spawns_get_closed_kinds_that_survive_their_code_and_stitches_round_trip() {
    let kinds = COMMANDS.map(|(program, words)| SpanKind::spawning(Command::new(program).args(words)));
    let decoded = CODED.map(|kind| SpanKind::try_from(KindCode::from(kind)));
    assert_eq!((kinds, decoded, SpanKind::try_from(KindCode::UNKNOWN)), (KINDS, CODED.map(Ok), Err(())));
    assert_eq!(STITCH.parse::<Stitch>().map(OsString::from), Ok(OsString::from(STITCH)));
}

#[test]
fn the_latest_trace_of_a_verb_is_read_back_and_files_past_the_count_or_the_bytes_are_pruned() {
    let dir = tempfile::tempdir().unwrap();
    let traces = TraceDir::new(dir.path().join(TRACES[0]));
    let [by_count, by_bytes] = RETAINED.map(|retained| serde_json::from_str::<Retention>(retained).unwrap());
    let kept = |index: usize, retention| {
        let start = Micros::new(u64::try_from(LAST - index).unwrap());
        let span = SpanRecord::new(SpanId::new(1, index), None, SpanKind::Command(VERBS[index]), start);
        traces.kept(TRACES[index].parse().unwrap(), &crate::Trace::new(vec![span]), retention).unwrap();
    };
    let latest = |verb| traces.latest(verb).unwrap().map(|trace| trace.started_by(Some(Verb::Hook)));
    assert_eq!(latest(None), None);
    (0..LAST).for_each(|index| kept(index, by_count));
    assert_eq!(latest(Some(Verb::Status)), None);
    kept(LAST, by_bytes);
    assert_eq!([latest(None), latest(Some(Verb::Gc)), latest(Some(Verb::Hook))], [Some(false), Some(false), None]);
}

#[test]
fn a_recording_spans_the_command_and_its_nested_work_and_stitches_only_meta_children() {
    let dir = tempfile::tempdir().unwrap();
    let recording = Recording::started(Verb::Status).unwrap();
    let spawned = SpanKind::Spawn(Meta).entered();
    std::thread::sleep(PAUSE);
    let stitches = [SpanKind::Spawn(Meta).stitch(), SpanKind::Spawn(Git).stitch()];
    let walked = Fs.entered();
    std::thread::sleep(PAUSE);
    drop((walked, spawned));
    recording.kept(&TraceDir::new(dir.path().to_path_buf()), Retention::default()).unwrap();
    let shown = TraceDir::new(dir.path().to_path_buf()).latest(Some(Verb::Status)).unwrap().unwrap().to_string();
    let [critical, untraced] = [0, 1].map(|part| shown.split(SECTIONS).nth(part).unwrap_or_default());
    assert!(stitches[0].is_some() && stitches[1].is_none(), "{shown}");
    assert!(critical.contains(WALKED) && untraced.contains(UNTRACED), "{shown}");
}
