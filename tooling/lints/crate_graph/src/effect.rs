use strum::Display;

use task::UsePath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Effect {
    Process,
    Files,
    Environment,
    Clock,
    Parsing,
}

impl Effect {
    #[must_use]
    pub fn touched_by(path: &UsePath) -> Option<Effect> {
        let spelled: Vec<String> = path.segments().iter().map(ToString::to_string).collect();
        let segments: Vec<&str> = spelled.iter().map(String::as_str).collect();
        match segments.as_slice() {
            ["std", "process", ..] | ["std", "os", "unix", "process", ..] => Some(Effect::Process),
            ["std", "fs", ..] => Some(Effect::Files),
            ["std", "env", ..] => Some(Effect::Environment),
            ["std", "time", .., "now"] | ["chrono", .., "now"] => Some(Effect::Clock),
            ["toml" | "serde_json", .., function] if function.starts_with(char::is_lowercase) => {
                Some(Effect::Parsing)
            }
            _ => None,
        }
    }
}
