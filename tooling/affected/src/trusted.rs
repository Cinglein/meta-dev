#[path = "oracle.rs"]
mod oracle;

use oracle::tests::Oracle;

#[test]
fn a_crate_is_tested_exactly_when_every_trusted_commit_changed_one_of_its_inputs() {
    assert!(Oracle::every().all(|oracle| oracle.agrees()));
}
