use std::io::{self, ErrorKind};
use std::os::unix::fs::{MetadataExt as _, symlink};
use std::path::{Path, PathBuf};

use super::Procfs;
use crate::request::local::local_clone::vetting::host::process_table::scan::Scan;

const ALIVE: &str = "alive";
const EXITED: &str = "exited";
const PLAIN: &str = "plain";
const NOT_A_DIR: &str = "plain/inner";
const LINKED: [(&str, &str); 3] = [("cwd", "/work"), ("root", "/"), ("fd/3", "/work/open.log")];
const OPENED: [&str; 3] = ["/work", "/", "/work/open.log"];
const UNREADABLE: &str = "exe";

fn excused(own: u32, process: &Path, kind: ErrorKind) -> Option<Scan> {
    Procfs { own }.excused(process, io::Error::from(kind)).ok()
}

fn planted(root: &Path) -> PathBuf {
    let process = root.join(ALIVE);
    std::fs::create_dir_all(process.join("fd")).unwrap();
    LINKED.iter().try_for_each(|(link, target)| symlink(target, process.join(link))).unwrap();
    process
}

#[test]
fn an_own_process_refused_is_uninspected_one_gone_or_foreign_is_skipped_and_any_other_failure_fails_closed() {
    let root = tempfile::tempdir().unwrap();
    let alive = planted(root.path());
    std::fs::write(root.path().join(PLAIN), "").unwrap();
    let own = std::fs::metadata(&alive).unwrap().uid();
    let foreign = own.wrapping_add(1);
    let cases = [
        (own, alive.clone(), ErrorKind::PermissionDenied, Some(Scan::Uninspected)),
        (own, alive.clone(), ErrorKind::NotFound, Some(Scan::Skipped)),
        (own, alive.clone(), ErrorKind::Other, None),
        (foreign, alive.clone(), ErrorKind::PermissionDenied, Some(Scan::Skipped)),
        (foreign, alive, ErrorKind::Other, Some(Scan::Skipped)),
        (own, root.path().join(EXITED), ErrorKind::Other, Some(Scan::Skipped)),
        (own, root.path().join(NOT_A_DIR), ErrorKind::Other, None),
    ];
    let judged = cases.each_ref().map(|(own, process, kind, _)| excused(*own, process, *kind));
    assert_eq!(judged, cases.map(|(.., expected)| expected));
}

#[test]
fn a_process_lists_its_links_and_open_files_and_a_missing_link_is_left_out_but_an_unreadable_one_fails() {
    let root = tempfile::tempdir().unwrap();
    let process = planted(root.path());
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
