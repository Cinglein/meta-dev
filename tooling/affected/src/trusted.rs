#[path = "oracle.rs"]
mod oracle;

use oracle::tests::Oracle;

#[test]
fn a_crate_is_reused_exactly_when_some_trusted_commit_leaves_its_inputs_identical() {
    assert!(Oracle::every().all(|oracle| oracle.agrees()));
}
