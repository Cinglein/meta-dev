use super::Entity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Concern {}

impl Entity for Concern {
    const PREFIX: &'static str = "concern";
}
