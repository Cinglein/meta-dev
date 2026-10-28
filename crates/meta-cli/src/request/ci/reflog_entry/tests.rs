use meta_golden::{GoldenFailure, Goldens, Recording};
use meta_run::GitArgv;

use super::ReflogEntry;

const SCENARIO: &str = "ci-session";

#[test]
fn every_recorded_reflog_walk_golden_view_is_its_entries() -> Result<(), GoldenFailure> {
    let recordings = Recording::all_in(&Goldens::workspace().dir().join(SCENARIO))?;
    let walks = recordings.iter().filter(|recording| recording.argv().calls(GitArgv::ReflogWalk));
    let parsed = |walk: &Recording| walk.out().lines().map(str::parse).collect::<Result<Vec<ReflogEntry>, ()>>();
    let viewed = walks.map(|walk| walk.viewed(&parsed(walk)));
    assert_ne!(viewed.collect::<Result<Vec<()>, _>>()?, []);
    Ok(())
}
