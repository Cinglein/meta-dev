use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::Disk;
use crate::request::local::local_clone::vetting::thaw::Thaw;

const FILE: &str = "tree/dir/file";
const LINKED: &str = "tree/linked";
const OUTSIDE: &str = "outside";
const TREE: &str = "tree";
const COMMON: &str = "common";
const WRITABLE: u32 = 0o640;
const FROZEN: u32 = 0o440;
const MODE_BITS: u32 = 0o777;

struct Planted(tempfile::TempDir);

impl Planted {
    fn new() -> Self {
        let planted = Self(tempfile::tempdir().unwrap());
        std::fs::create_dir_all(planted.located(FILE).parent().unwrap()).unwrap();
        [FILE, LINKED]
            .iter()
            .try_for_each(|name| {
                std::fs::write(planted.located(name), TREE)?;
                std::fs::set_permissions(planted.located(name), Permissions::from_mode(WRITABLE))
            })
            .unwrap();
        std::fs::hard_link(planted.located(LINKED), planted.located(OUTSIDE)).unwrap();
        planted
    }

    fn located(&self, name: &str) -> PathBuf {
        self.0.path().join(name)
    }

    fn mode(&self, name: &str) -> u32 {
        std::fs::metadata(self.located(name)).unwrap().permissions().mode() & MODE_BITS
    }
}

#[test]
fn freezing_records_the_old_modes_first_skips_hard_links_and_thawing_restores_and_forgets_them() {
    let planted = Planted::new();
    let plan = Thaw::planned(Path::new(TREE), Thaw::ledger_of(&planted.located(TREE), &planted.located(COMMON)));
    let thaw = Disk::frozen(&planted.located(TREE), plan).unwrap();
    let ledger = std::fs::read_to_string(thaw.ledger()).unwrap();
    let frozen = [planted.mode(FILE), planted.mode(LINKED)];
    Disk::thawed(&thaw);
    assert!(ledger.contains(FILE) && !ledger.contains(LINKED) && !thaw.ledger().exists(), "{ledger}");
    assert_eq!([frozen, [planted.mode(FILE), planted.mode(LINKED)]], [[FROZEN, WRITABLE], [WRITABLE, WRITABLE]]);
}
