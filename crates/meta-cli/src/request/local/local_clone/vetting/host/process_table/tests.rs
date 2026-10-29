use std::fs::File;
use std::path::PathBuf;
use std::process::{Child, Command};

use super::ProcessTable;

const SLEEP: &str = "sleep";
const SECONDS: &str = "30";
const DIRS: [&str; 3] = ["cwd", "held", "cwd-sibling"];
const FILE: &str = "open.log";
const BUSY: [bool; 3] = [false, false, true];
const EXITED: [bool; 3] = [true, false, true];

struct Sleeper(Child);

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
    let busy = ProcessTable::occupancy(&dirs);
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let exited = ProcessTable::occupancy(&dirs);
    assert_eq!(dirs.each_ref().map(|dir| busy.vacant(dir).is_ok()), BUSY, "{busy:?}");
    assert_eq!(dirs.each_ref().map(|dir| exited.vacant(dir).is_ok()), EXITED, "{exited:?}");
}
