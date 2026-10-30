use super::Entity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Project {}

impl Entity for Project {
    const PREFIX: &'static str = "project";
}
