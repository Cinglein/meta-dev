pub trait Entity {
    const PREFIX: &'static str;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Concern {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Issue {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Task {}

impl Entity for Concern {
    const PREFIX: &'static str = "concern";
}

impl Entity for Issue {
    const PREFIX: &'static str = "issue";
}

impl Entity for Task {
    const PREFIX: &'static str = "task";
}
