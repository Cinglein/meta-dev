mod pattern;

use meta_schema::TopDivision;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use super::scope_kind::ScopeKind;
use pattern::Pattern;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Domain {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tests: Vec<Pattern>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    workflows: Vec<Pattern>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    crates: Vec<Pattern>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    paths: Vec<Pattern>,
}

impl Domain {
    #[must_use]
    pub fn owned_by(top: TopDivision) -> Domain {
        let everything = || vec![Pattern::everything()];
        match top {
            TopDivision::FeaturesAndMaintenance => Domain {
                crates: everything(),
                paths: everything(),
                ..Domain::default()
            },
            TopDivision::PerformanceAndCost => Domain {
                tests: everything(),
                workflows: everything(),
                ..Domain::default()
            },
            TopDivision::ArchitectureAndPlanning => Domain::default(),
        }
    }

    #[must_use]
    pub fn scope(&self, kind: ScopeKind) -> &[Pattern] {
        match kind {
            ScopeKind::Tests => &self.tests,
            ScopeKind::Workflows => &self.workflows,
            ScopeKind::Crates => &self.crates,
            ScopeKind::Paths => &self.paths,
        }
    }

    #[must_use]
    pub fn declares(&self, kind: ScopeKind) -> bool {
        !self.scope(kind).is_empty()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        ScopeKind::iter().all(|kind| !self.declares(kind))
    }

    #[must_use]
    pub fn is_strictly_within(&self, outer: &Domain) -> bool {
        ScopeKind::iter().all(|kind| outer.covers(self, kind))
            && !ScopeKind::iter().all(|kind| self.covers(outer, kind))
    }

    fn covers(&self, inner: &Domain, kind: ScopeKind) -> bool {
        inner
            .scope(kind)
            .iter()
            .all(|pattern| pattern.is_covered_by(self.scope(kind)))
    }
}
