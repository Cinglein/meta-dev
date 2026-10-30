use super::Entity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lease {}

impl Entity for Lease {
    const PREFIX: &'static str = "lease";
}
