use std::io::{self, ErrorKind};
use std::os::unix::fs::{MetadataExt as _, symlink};
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::Procfs;
use crate::request::local::local_clone::vetting::host::process_table::scan::Scan;

const ALIVE: &str = "alive";
const EXITED: &str = "exited";
const PLAIN: &str = "plain";
const NOT_A_DIR: &str = "plain/inner";
const LINKED: [(&str, &str); 3] = [("cwd", "/work"), ("root", "/"), ("fd/3", "/work/open.log")];
const OPENED: [&str; 3] = ["/work", "/", "/work/open.log"];
const UNREADABLE: &str = "exe";

struct FakeProc(TempDir);

impl FakeProc {
    fn planted() -> Self {
        let root = tempfile::tempdir().unwrap();
        let process = root.path().join(ALIVE);
        std::fs::create_dir_all(process.join(Procfs::FDS)).unwrap();
        LINKED.iter().try_for_each(|(link, target)| symlink(target, process.join(link))).unwrap();
        std::fs::write(root.path().join(PLAIN), "").unwrap();
        Self(root)
    }

    fn located(&self, name: &str) -> PathBuf {
        self.0.path().join(name)
    }

    fn excused(&self, own: u32, name: &str, kind: ErrorKind) -> Option<Scan> {
        Procfs { own }.excused(&self.located(name), io::Error::from(kind)).ok()
    }
}

#[test]
fn an_own_process_refused_is_uninspected_one_gone_or_foreign_is_skipped_and_any_other_failure_fails_closed() {
    let fake = FakeProc::planted();
    let own = std::fs::metadata(fake.located(ALIVE)).unwrap().uid();
    let foreign = own.wrapping_add(1);
    let cases = [
        (own, ALIVE, ErrorKind::PermissionDenied, Some(Scan::Uninspected)),
        (own, ALIVE, ErrorKind::NotFound, Some(Scan::Skipped)),
        (own, ALIVE, ErrorKind::Other, None),
        (foreign, ALIVE, ErrorKind::PermissionDenied, Some(Scan::Skipped)),
        (foreign, ALIVE, ErrorKind::Other, Some(Scan::Skipped)),
        (own, EXITED, ErrorKind::Other, Some(Scan::Skipped)),
        (own, NOT_A_DIR, ErrorKind::Other, None),
    ];
    let judged = cases.each_ref().map(|(own, name, kind, _)| fake.excused(*own, name, *kind));
    assert_eq!(judged, cases.map(|(.., expected)| expected));
}

#[test]
fn a_process_lists_its_links_and_open_files_and_a_missing_link_is_left_out_but_an_unreadable_one_fails() {
    let fake = FakeProc::planted();
    let process = fake.located(ALIVE);
    let opened = Procfs::opened(&process).unwrap();
    std::fs::write(process.join(UNREADABLE), "").unwrap();
    let unreadable = Procfs::opened(&process).map_err(|failure| failure.kind());
    assert_eq!(opened, OPENED.map(PathBuf::from));
    assert_eq!(unreadable, Err(ErrorKind::InvalidInput));
}

#[test]
fn the_table_is_read_exactly_where_proc_exists() {
    assert_eq!(Procfs::held().is_ok(), Path::new(Procfs::SELF).exists());
}
