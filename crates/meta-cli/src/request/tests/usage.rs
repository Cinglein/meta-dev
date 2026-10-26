use super::PROGRAM;
use crate::request::Request;

const HOOK_LINE: &str = "\n  hook session-start|pre-tool-use|post-tool-use|stop   (Claude Code hook JSON on stdin)\n";
const HEARTBEAT_LINE: &str = "\n  heartbeat tick|flags|plist   (";

#[test]
fn the_usage_text_keeps_a_space_between_each_command_and_its_arguments() {
    let refused = Request::try_from([PROGRAM, "bogus"].map(String::from).to_vec()).unwrap_err();
    assert!(refused.to_string().contains(HOOK_LINE) && refused.to_string().contains(HEARTBEAT_LINE));
}
