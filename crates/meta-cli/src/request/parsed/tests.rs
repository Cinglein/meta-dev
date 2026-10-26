use super::Parsed;

const TASK: &str = "task-144fe5a44d4971a5";
const PREVIOUS: &str = "alice";

#[test]
fn a_from_option_names_the_previous_claimant_and_its_absence_names_none() {
    let args = |words: &[&str]| words.iter().map(ToString::to_string).collect::<Vec<String>>();
    let given = Parsed::try_from(args(&[TASK, "--from", PREVIOUS]).as_slice()).unwrap();
    let absent = Parsed::try_from(args(&[TASK]).as_slice()).unwrap();
    assert_eq!(given.previous_claimant(), Some(PREVIOUS.parse().unwrap()));
    assert_eq!(absent.previous_claimant(), None);
}
