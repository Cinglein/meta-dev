use strum::{Display, EnumIter};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ScopeKind {
    Tests,
    Workflows,
    Crates,
    Paths,
}
