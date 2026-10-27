use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ownership {
    Agent,
    Attended,
    Unsure,
    Outside,
}

impl fmt::Display for Ownership {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Agent => "agent",
            Self::Attended => "agent, session alive",
            Self::Unsure => "unsure",
            Self::Outside => "not agent work",
        })
    }
}
