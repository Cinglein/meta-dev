use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveSubject(String);

impl MoveSubject {
    const ONLY_MOVES_FIRST: [&'static str; 3] = ["checkout:", "reset:", "clone:"];
    const ONLY_MOVES_ANYWHERE: [&'static str; 4] = ["Fast-forward", "(start)", "(finish)", "(abort)"];

    pub fn creates(&self) -> bool {
        !Self::ONLY_MOVES_FIRST.iter().any(|kind| self.0.starts_with(kind))
            && !Self::ONLY_MOVES_ANYWHERE.iter().any(|kind| self.0.contains(kind))
    }
}

impl FromStr for MoveSubject {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self(text.to_owned()))
    }
}
