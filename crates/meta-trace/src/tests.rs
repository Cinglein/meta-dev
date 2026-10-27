use std::cell::OnceCell;
use std::process::Command;

use crate::{CommandWord, Retention, SpanKind, SpawnSpan, Trace, TraceSwitch};

const ARGS: [&str; 3] = ["meta", "sta/tus!", "--flags"];
const UNSPAWNED: [(&str, &[&str]); 3] = [
    ("git", &["-C", "/private/path", "fetch", "", "--quiet", "origin", "task-00a1"]),
    ("/usr/bin/gh", &[]),
    ("cargo", &["fetch"]),
];
const DETACHED: &str = "true";
const EXITING: (&str, [&str; 2]) = ("sh", ["-c", "exit 3"]);
const SEEN: [&str; 6] = [
    r#""name":"status","cat":"command""#,
    r#""args":{"program":"git","args":"fetch origin","network":true,"exit":"unspawned"}"#,
    r#""args":{"program":"gh","args":"","network":true,"exit":"unspawned"}"#,
    r#""args":{"program":"cargo","args":"fetch","network":false,"exit":"unspawned"}"#,
    r#""name":"git fetch origin","cat":"spawn","#,
    r#""args":{"program":"sh","args":"","network":false,"exit":"exit status: 3"}},{"name":"walk","cat":"fs","#,
];
const WALK: &str = "walk";
const DETACHED_SEEN: &str = r#""args":{"program":"true","args":"","network":false,"exit":"detached"}"#;
const TRACES: &str = "traces";
const SWITCHES: [&str; 4] = ["on", "", "off", "x"];
const SWITCHED: [Result<TraceSwitch, bool>; 4] =
    [Ok(TraceSwitch::On), Ok(TraceSwitch::Off), Ok(TraceSwitch::Off), Err(true)];
const UNKNOWN: &str = "META_TRACE must be on or off";
const COUNTED: &str = "spawns git 1\n";
const COMMAND: &str = "ms command status";
const INSTANT: &str = " 0.000ms";
const FIXTURE: &str = r#"{"traceEvents": [
{"name": "git fetch origin", "cat": "spawn", "ph": "X", "ts": 0, "dur": 3000, "pid": 1, "tid": 1, "args": {"program": "git", "network": true}},
{"name": "lsof", "cat": "spawn", "ph": "X", "ts": 3100, "dur": 800, "pid": 1, "tid": 1, "args": {"program": "lsof"}},
{"name": "walk", "cat": "fs", "ph": "X", "ts": 3000, "dur": 1000, "pid": 1, "tid": 1},
{"name": "gh pr list", "cat": "spawn", "ph": "X", "ts": 4000, "dur": 4800, "pid": 1, "tid": 1, "args": {"program": "gh", "network": true}},
{"name": "git rev-parse", "cat": "spawn", "ph": "X", "ts": 8900, "dur": 0, "pid": 1, "tid": 1, "args": {"program": "git"}},
{"name": "git status", "cat": "spawn", "ph": "X", "ts": 9000, "dur": 1000, "pid": 1, "tid": 1, "args": {"program": "git", "network": false}},
{"name": "status", "cat": "command", "ph": "X", "ts": 0, "dur": 10000, "pid": 1, "tid": 1}]}"#;
const SUMMARY: &str = "critical path:
    10.000ms command status
       3.000ms spawn git fetch origin
       1.000ms fs walk
         0.800ms spawn lsof
       4.800ms spawn gh pr list
       0.000ms spawn git rev-parse
       1.000ms spawn git status
top 5 spans:
    10.000ms command status
     4.800ms spawn gh pr list
     3.000ms spawn git fetch origin
     1.000ms fs walk
     1.000ms spawn git status
spawns gh 1
spawns git 3
spawns lsof 1
network gh 1
network git 1
";

#[test]
fn a_recorded_run_holds_a_redacted_span_per_spawn_and_nothing_when_off() {
    let seen = OnceCell::new();
    let word = CommandWord::from(&ARGS.map(String::from)[..]);
    let spawning = || {
        let _ = UNSPAWNED.map(|(program, args)| SpawnSpan::opened(Command::new(program).args(args)).ended(None));
        SpawnSpan::detached(&mut Command::new(DETACHED)).and_then(|mut child| child.wait()).unwrap();
        let _walk = SpanKind::Fs.entered(&WALK);
        SpawnSpan::status_of(Command::new(EXITING.0).args(EXITING.1)).unwrap().code()
    };
    let keep = |trace| seen.set(serde_json::to_string(&trace).unwrap()).unwrap();
    let exited = TraceSwitch::On.recorded(&word, spawning, keep);
    let seen = seen.get().unwrap();
    assert!(SEEN.iter().all(|fragment| seen.contains(fragment)) && seen.contains(DETACHED_SEEN));
    assert_eq!((exited, TraceSwitch::Off.recorded(&word, || 0, |_| panic!())), (Some(3), 0));
    let shown = serde_json::from_str::<Trace>(seen).unwrap().to_string();
    let root = shown.lines().nth(1).unwrap_or_default();
    assert!(shown.contains(COUNTED) && root.ends_with(COMMAND) && !root.contains(INSTANT));
}

#[test]
fn traces_are_kept_oldest_first_and_pruned_to_the_retention_and_the_switch_reads_on_or_off() {
    let dir = tempfile::tempdir().unwrap();
    let traces = dir.path().join(TRACES);
    let word = CommandWord::from(&ARGS.map(String::from)[..]);
    let kept: Retention = serde_json::from_str("2").unwrap();
    let empty = Trace::latest(&traces).unwrap();
    let paths: Vec<_> = (0..3).map(|_| Trace::default().kept(&traces, &word, kept).unwrap()).collect();
    std::fs::write(traces.join(TRACES), "").unwrap();
    let left = std::fs::read_dir(&traces).unwrap().count();
    assert!(empty.is_none() && paths.iter().all(|path| path.to_string_lossy().ends_with("-status.json")));
    let latest = Trace::latest(&traces).unwrap();
    assert_eq!((left, paths[2].exists(), paths[0].exists(), latest), (3, true, false, Some(Trace::default())));
    let switches = SWITCHES.map(|value| value.parse::<TraceSwitch>().map_err(|failure| failure.to_string() == UNKNOWN));
    assert_eq!((switches, Retention::default()), (SWITCHED, serde_json::from_str("20").unwrap()));
}

#[test]
fn the_summary_walks_the_critical_path_and_counts_spawns_and_network_calls() {
    let trace: Trace = serde_json::from_str(FIXTURE).unwrap();
    assert_eq!(trace.to_string(), SUMMARY);
}
