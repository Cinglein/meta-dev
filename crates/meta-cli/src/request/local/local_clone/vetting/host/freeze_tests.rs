use meta_door::kinds::{Permissions, PermissionsExt};
use std::path::{Path, PathBuf};

use meta_door::{Change, Door, Pair};

use super::Host;
use crate::request::local::local_clone::vetting::inspection::Inspection;
use crate::request::local::local_clone::vetting::thaw::Thaw;
use crate::request::tests::Repository;

const FILE: &str = "tree/dir/file";
const LINKED: &str = "tree/linked";
const OUTSIDE: &str = "outside";
const TREE: &str = "tree";
const WRITABLE: u32 = 0o640;
const FROZEN: u32 = 0o440;
const MODE_BITS: u32 = 0o777;

struct Planted(Repository);

impl Planted {
    fn new() -> Self {
        let planted = Self(Repository::new());
        Door::changed(planted.located(FILE).parent().unwrap(), Change::DirAll).unwrap();
        [FILE, LINKED]
            .iter()
            .try_for_each(|name| {
                Door::write(planted.located(name), TREE)?;
                Door::changed(planted.located(name), Change::Permissions(Permissions::from_mode(WRITABLE)))
            })
            .unwrap();
        Door::paired(planted.located(LINKED), planted.located(OUTSIDE), Pair::HardLink).unwrap();
        planted
    }

    fn located(&self, name: &str) -> PathBuf {
        self.0.path().join(name)
    }

    fn mode(&self, name: &str) -> u32 {
        self.located(name).metadata().unwrap().permissions().mode() & MODE_BITS
    }
}

#[test]
fn freezing_records_the_old_modes_first_skips_hard_links_and_thawing_restores_and_forgets_them() {
    let planted = Planted::new();
    let plan = Thaw::planned(Path::new(TREE), Thaw::ledger_of(&planted.located(TREE), planted.0.store().common()));
    let thaw = Host::default().frozen(&planted.located(TREE), plan).unwrap();
    let ledger = Door::read(thaw.ledger().unwrap()).unwrap();
    let frozen = [planted.mode(FILE), planted.mode(LINKED)];
    Host::default().thawed(&thaw);
    assert!(ledger.contains(FILE) && !ledger.contains(LINKED) && !thaw.ledger().unwrap().path().exists(), "{ledger}");
    assert_eq!([frozen, [planted.mode(FILE), planted.mode(LINKED)]], [[FROZEN, WRITABLE], [WRITABLE, WRITABLE]]);
}
