use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::tally::Tally;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reduction<Attribute> {
    Equal(Attribute),
    Meet(BTreeSet<Attribute>),
}

impl<Attribute: Ord + Clone> Reduction<Attribute> {
    #[must_use]
    pub fn found<Object: Ord>(
        attribute: &Attribute,
        extent: &BTreeSet<Object>,
        extents: &BTreeMap<Attribute, BTreeSet<Object>>,
    ) -> Option<Reduction<Attribute>> {
        let equal = extents.iter().find(|(other, wider)| *other < attribute && *wider == extent);
        let wider: Vec<&BTreeSet<Object>> =
            extents.values().filter(|wider| extent.is_subset(wider) && *wider != extent).collect();
        let minimal: BTreeSet<Attribute> = extents
            .iter()
            .filter(|(_, above)| {
                wider.contains(above) && !wider.iter().any(|between| between.is_subset(above) && between != above)
            })
            .map(|(other, _)| other.clone())
            .collect();
        let bases: Vec<&BTreeSet<Object>> = minimal.iter().filter_map(|other| extents.get(other)).collect();
        let common = bases.first().map(|first| Self::within_all(first, &bases));
        let meet = Some(minimal)
            .filter(|minimal| Tally::counting(minimal).reaches(Tally::PAIR))
            .filter(|_| common.is_some_and(|common| common.into_iter().eq(extent.iter())));
        equal.map(|(other, _)| Reduction::Equal(other.clone())).or_else(|| meet.map(Reduction::Meet))
    }

    fn within_all<'objects, Object: Ord>(
        first: &'objects BTreeSet<Object>,
        bases: &[&BTreeSet<Object>],
    ) -> Vec<&'objects Object> {
        first.iter().filter(|object| bases.iter().all(|base| base.contains(object))).collect()
    }
}

impl<Attribute: fmt::Display> fmt::Display for Reduction<Attribute> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reduction::Equal(other) => write!(formatter, "same implementors as {other}"),
            Reduction::Meet(others) => {
                let names: Vec<String> = others.iter().map(ToString::to_string).collect();
                write!(formatter, "implementors are exactly those of {}", names.join(" + "))
            }
        }
    }
}
