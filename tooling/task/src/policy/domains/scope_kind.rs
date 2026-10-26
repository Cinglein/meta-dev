use meta_schema::TopDivision;
use strum::{Display, EnumIter};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ScopeKind {
    Tests,
    Workflows,
    Crates,
    Paths,
}

impl ScopeKind {
    #[must_use]
    pub const fn owner(self) -> TopDivision {
        match self {
            ScopeKind::Tests | ScopeKind::Workflows => TopDivision::PerformanceAndCost,
            ScopeKind::Crates | ScopeKind::Paths => TopDivision::FeaturesAndMaintenance,
        }
    }
}
