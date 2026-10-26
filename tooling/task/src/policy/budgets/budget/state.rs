use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
#[strum(serialize_all = "snake_case")]
pub enum State {
    Ok,
    Warn,
    Over,
}
