use chrono::TimeDelta;
use meta_golden::Golden;
use meta_log::MemoryLog;

use super::{Beat, SETTINGS, Scene};
use crate::request::heartbeat::{LatestFlags, Tick};

const ZERO: TimeDelta = TimeDelta::seconds(0);
const AGES: [(Scene, &[TimeDelta]); 2] = [
    (Scene::Orphaned, &[TimeDelta::seconds(60), TimeDelta::seconds(900), TimeDelta::seconds(901)]),
    (
        Scene::Quiet,
        &[TimeDelta::seconds(1), TimeDelta::seconds(-5), TimeDelta::seconds(-60), TimeDelta::seconds(-3600)],
    ),
];

#[test]
fn flags_show_the_latest_snapshot_and_its_age_and_a_missing_stale_or_far_future_snapshot_is_a_flag() {
    let settings = Beat::settings(SETTINGS).unwrap();
    let never = tempfile::tempdir().unwrap();
    let never = LatestFlags::answer(never.path(), &settings, Beat::after(ZERO)).unwrap();
    let aged = AGES.iter().flat_map(|(scene, ages)| {
        let git_dir = tempfile::tempdir().unwrap();
        Tick::answer(git_dir.path(), &MemoryLog::default(), scene.answering(), Beat::pulse(ZERO)).unwrap();
        ages.iter()
            .map(|age| LatestFlags::answer(git_dir.path(), &settings, Beat::after(*age)).unwrap())
            .collect::<Vec<_>>()
    });
    let shown: Vec<_> = std::iter::once(never).chain(aged).collect();
    Golden::for_test(module_path!(), "flags").debugged(&shown).unwrap();
}
