use super::LevelName;

const ACCEPTED: [&str; 3] = ["minor", "p1", "wrong-turn"];
const REFUSED: [&str; 5] = ["", "12", "Major", "wrong_turn", "-"];
const STORED_UPPER: &str = r#""Major""#;

#[test]
fn a_level_name_is_lowercase_letters_digits_and_hyphens_with_a_letter() {
    let accepted = ACCEPTED.map(|name| name.parse::<LevelName>().map(|level| level.to_string()).ok());
    assert_eq!(accepted, ACCEPTED.map(|name| Some(name.to_owned())));
    assert!(REFUSED.iter().all(|name| name.parse::<LevelName>().is_err()));
    assert!(serde_json::from_str::<LevelName>(STORED_UPPER).is_err());
}
