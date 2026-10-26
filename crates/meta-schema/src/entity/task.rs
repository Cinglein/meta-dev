use super::Entity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Task {}

impl Entity for Task {
    const PREFIX: &'static str = "task";
}
