use meta_door::kinds::{Permissions, PermissionsExt};
use std::path::{Path, PathBuf};

use super::deletable::Deletable;
use super::fake_host::{FakeHost, PATH};
use super::thaw::{Thaw, Thawed};

const QUARANTINE: &str = "/trees/.gc-task-00000000000000b1-1x0";
const COMMON: &str = "/clone/.git";
const LEDGER: &str = "/clone/.git/meta-local/quarantine/.gc-task-00000000000000b1-1x0.json";
const FILE: &str = "/trees/.gc-task-00000000000000b1-1x0/meta.toml";
const WRITABLE: u32 = 0o644;
const STEPS: [&str; 3] = [
    "thawed /trees/.gc-task-00000000000000b1-1x0/meta.toml",
    "worktree unlock /trees/.gc-task-00000000000000b1-1x0",
    "worktree move /trees/.gc-task-00000000000000b1-1x0 /trees/task-00000000000000b1",
];

#[test]
fn a_stranded_quarantine_is_restored_from_its_ledger_and_without_one_nothing_moves() {
    let thaw = Thaw::planned(Path::new(PATH), Thaw::ledger_of(Path::new(QUARANTINE), Path::new(COMMON)))
        .holding(vec![Thawed::new(PathBuf::from(FILE), &Permissions::from_mode(WRITABLE))]);
    let mut host = FakeHost::clean();
    host.text_is(LEDGER, &serde_json::to_string(&thaw).unwrap());
    let restored = Deletable::unstranded(Path::new(QUARANTINE), Path::new(COMMON), &host);
    let calls = host.calls();
    assert!(restored.is_ok_and(|origin| origin == Path::new(PATH)) && thaw.origin() == Path::new(PATH));
    assert!(
        STEPS.iter().zip(calls.iter()).all(|(step, call)| call.ends_with(step)) && calls.len() == STEPS.len(),
        "{calls:?}"
    );
    let empty = FakeHost::clean();
    assert!(
        Deletable::unstranded(Path::new(QUARANTINE), Path::new(COMMON), &empty).is_err() && empty.calls().is_empty()
    );
}
