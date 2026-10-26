#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveSubject(String);

impl MoveSubject {
    const ONLY_MOVES_FIRST: [&'static str; 2] = ["checkout:", "reset:"];
    const ONLY_MOVES_ANYWHERE: [&'static str; 4] = ["Fast-forward", "(start)", "(finish)", "(abort)"];

    pub fn new(subject: &str) -> Self {
        Self(subject.to_owned())
    }

    pub fn creates(&self) -> bool {
        !Self::ONLY_MOVES_FIRST.iter().any(|kind| self.0.starts_with(kind))
            && !Self::ONLY_MOVES_ANYWHERE.iter().any(|kind| self.0.contains(kind))
    }
}
