use meta_golden::Golden;

use super::PROGRAM;
use crate::request::Request;

#[test]
fn the_usage_text_keeps_a_space_between_each_command_and_its_arguments() {
    let refused = Request::try_from([PROGRAM, "bogus"].map(String::from).to_vec()).unwrap_err();
    Golden::for_test(module_path!(), "usage").shown(&refused).unwrap();
}
