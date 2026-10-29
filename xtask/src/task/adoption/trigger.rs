use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub enum Trigger {
    #[strum(serialize = "pull_request")]
    PullRequest,
    #[strum(serialize = "push or schedule")]
    Main,
}

impl Trigger {
    pub fn events(self) -> &'static [&'static str] {
        match self {
            Trigger::PullRequest => &["pull_request"],
            Trigger::Main => &["push", "schedule"],
        }
    }
}
