use std::os::unix::fs::PermissionsExt;

use super::Host;
use crate::request::local::local_clone::vetting::inspection::Inspection;

const FILE: &str = "dir/file";
const DIR: &str = "dir";
const WRITABLE: u32 = 0o640;
const FROZEN: u32 = 0o440;
const MODE_BITS: u32 = 0o777;

#[test]
fn freezing_drops_every_write_bit_on_files_and_thawing_restores_the_old_modes() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join(DIR)).unwrap();
    std::fs::write(root.path().join(FILE), DIR).unwrap();
    std::fs::set_permissions(root.path().join(FILE), std::fs::Permissions::from_mode(WRITABLE)).unwrap();
    let mode = || std::fs::metadata(root.path().join(FILE)).unwrap().permissions().mode() & MODE_BITS;
    let thaw = Host::default().frozen(root.path()).unwrap();
    let frozen = mode();
    Host::default().thawed(&thaw);
    assert_eq!([frozen, mode()], [FROZEN, WRITABLE]);
}
