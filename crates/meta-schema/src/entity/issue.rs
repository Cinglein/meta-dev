use super::Entity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Issue {}

impl Entity for Issue {
    const PREFIX: &'static str = "issue";
}
