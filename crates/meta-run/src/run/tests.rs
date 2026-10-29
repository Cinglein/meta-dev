use std::time::{Duration, Instant};

use meta_trace::{Recording, Stitch, Verb};

use crate::env_policy::EnvPolicy;
use crate::run::Run;

const SHELL: &str = "/bin/sh";
const META: &str = "meta";
const SHOWN: [&str; 2] = ["-c", "printf %s \"$META_TRACE_PARENT\""];
const PATIENT: Duration = Duration::from_secs(5);

#[test]
fn a_meta_child_is_told_its_trace_and_parent_span_even_with_no_inherited_variables() {
    let _recording = Recording::started(Verb::Other).unwrap();
    let dir = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(SHELL, dir.path().join(META)).unwrap();
    let shown = Run::new(dir.path().join(META), EnvPolicy::Only(&[])).args(SHOWN).stdout(Instant::now() + PATIENT);
    assert!(shown.as_deref().is_ok_and(|shown| shown.parse::<Stitch>().is_ok()), "{shown:?}");
}
