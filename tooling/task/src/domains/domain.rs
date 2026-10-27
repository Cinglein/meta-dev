mod pattern;

use std::path::Path;

use meta_schema::TopDivision;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use super::extent::Extent;
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
        let owned = |kind: ScopeKind| {
            (kind.owner() == top)
                .then(Pattern::everything)
                .into_iter()
                .collect()
        };
        Domain {
            tests: owned(ScopeKind::Tests),
            workflows: owned(ScopeKind::Workflows),
            crates: owned(ScopeKind::Crates),
            paths: owned(ScopeKind::Paths),
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
    pub fn extent(&self, kind: ScopeKind) -> Extent {
        match self.scope(kind) {
            [] => Extent::Undeclared,
            patterns if patterns.contains(&Pattern::everything()) => Extent::Whole,
            _ => Extent::Partial,
        }
    }

    #[must_use]
    pub fn holds(&self, path: &Path) -> bool {
        path.to_str()
            .is_some_and(|name| self.paths.iter().any(|pattern| pattern.matches(name)))
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        ScopeKind::iter().all(|kind| self.extent(kind) == Extent::Undeclared)
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
