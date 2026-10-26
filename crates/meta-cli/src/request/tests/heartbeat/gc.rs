use super::Beat;

const FREE: &str = "[heartbeat]\ngc = \"free\"";
const DRY_RUN: &str = "[heartbeat]\ngc = \"dry-run\"";
const UNKNOWN: &str = "[heartbeat]\ngc = \"sometimes\"";

#[test]
fn the_heartbeat_gc_switch_takes_dry_run_or_free_and_nothing_else() {
    let read = [FREE, DRY_RUN, UNKNOWN].map(|text| Beat::settings(text).is_ok());
    assert_eq!(read, [true, true, false]);
}
