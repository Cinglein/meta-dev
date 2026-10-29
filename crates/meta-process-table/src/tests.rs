use std::fs::File;
use std::path::PathBuf;
use std::process::{Child, Command};

use super::{Held, ProcessTable, Scan};

const SLEEP: &str = "sleep";
const SECONDS: &str = "30";
const DIRS: [&str; 3] = ["cwd", "held", "cwd-sibling"];
const FILE: &str = "open.log";
const BUSY: [bool; 3] = [true, true, false];
const EXITED: [bool; 3] = [false, true, false];
const OPENED: [&str; 2] = ["/work/a", "/elsewhere/b"];
const SCANNED: [&str; 2] = ["/work", "/else"];
const UNINSPECTED: &str = "2 processes could not be inspected\n";

struct Sleeper(Child);

impl Sleeper {
    fn occupied(dirs: &[PathBuf; 3]) -> [bool; 3] {
        let under = ProcessTable::held().unwrap().under(dirs);
        dirs.each_ref().map(|dir| under.iter().any(|path| path.starts_with(dir)))
    }
}

impl Drop for Sleeper {
    fn drop(&mut self) {
        drop(self.0.kill());
        drop(self.0.wait());
    }
}

#[test]
fn a_child_cwd_or_an_open_file_under_a_dir_occupies_it_and_an_exited_child_does_not() {
    let root = tempfile::tempdir().unwrap();
    let dirs: [PathBuf; 3] = DIRS.map(|name| std::fs::canonicalize(root.path()).unwrap().join(name));
    dirs.iter().try_for_each(std::fs::create_dir).unwrap();
    let [cwd, held, _] = &dirs;
    let _open = File::create(held.join(FILE)).unwrap();
    let mut child = Sleeper(Command::new(SLEEP).arg(SECONDS).current_dir(cwd).spawn().unwrap());
    let busy = Sleeper::occupied(&dirs);
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    assert_eq!((busy, Sleeper::occupied(&dirs)), (BUSY, EXITED));
}

#[test]
fn a_table_keeps_the_opened_paths_under_the_scanned_dirs_and_counts_each_uninspected_process() {
    let opened = Scan::Opened(OPENED.map(PathBuf::from).to_vec());
    let held: Held = [Scan::Uninspected, Scan::Skipped, opened, Scan::Uninspected].into_iter().collect();
    let shown = held.uninspected().to_string();
    let under = held.under(&SCANNED.map(PathBuf::from));
    assert_eq!((shown.as_str(), under), (UNINSPECTED, vec![PathBuf::from(OPENED[0])]));
}
