use meta_golden::Golden;

use super::PROGRAM;
use crate::request::Request;

const RESTORE: [&str; 4] = [PROGRAM, "gc", "restore", "/trees/.gc-task-1x0"];
const PLANNED: [&str; 3] = [PROGRAM, "gc", "--dry-run"];
const OTHER: [&str; 4] = [PROGRAM, "gc", "forget", "/trees/.gc-task-1x0"];

#[test]
fn gc_restore_names_a_quarantine_and_anything_else_is_a_sweep() {
    let parsed = [&RESTORE[..], &PLANNED[..], &OTHER[..]]
        .map(|words| Request::try_from(words.iter().map(|word| (*word).to_owned()).collect::<Vec<_>>()).unwrap());
    Golden::for_test(module_path!(), "parsed").debugged(&parsed).unwrap();
}
