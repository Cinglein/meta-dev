use std::collections::BTreeSet;
use std::fmt;

use meta_schema::{Division, Placement};
use meta_severity::{Scales, SeverityFilter};
use meta_state::{RatedItem, State};

#[derive(Clone, Copy, Debug)]
pub struct SeverityTally<'view> {
    state: &'view State,
    scales: &'view Scales,
}

impl<'view> SeverityTally<'view> {
    const ROOT: &'static str = "root";

    #[must_use]
    pub fn new(state: &'view State, scales: &'view Scales) -> Self {
        Self { state, scales }
    }

    fn counts(&self, division: &Division, items: &[RatedItem]) -> String {
        let levels = self.scales.scale(division).map(|scale| scale.levels().to_vec()).unwrap_or_default();
        let filters = levels.iter().map(|level| SeverityFilter::AtLevel(level.name().clone()));
        let counted = filters.chain([SeverityFilter::Unrated]).map(|filter| {
            let count = items.iter().filter_map(|item| filter.admitted(self.scales, division, item)).count();
            format!("{filter} {count}")
        });
        counted.collect::<Vec<String>>().join(", ")
    }

    fn placed(items: &[RatedItem], placement: &Placement) -> String {
        let ratings: BTreeSet<String> = items.iter().map(|item| item.rating().to_string()).collect();
        let shown = ratings.iter().filter_map(|rating| {
            let rated =
                items.iter().filter(|item| item.placement() == placement && item.rating().to_string() == *rating);
            let ids: Vec<String> = rated.map(|item| item.target().to_string()).collect();
            ids.first().map(|_| format!("\n  {rating}: {}", ids.join(" ")))
        });
        shown.collect()
    }
}

impl fmt::Display for SeverityTally<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items = self.state.open_items();
        let divisions: BTreeSet<Division> =
            items.iter().filter_map(|item| item.placement().division()).flat_map(Division::lineage).collect();
        writeln!(formatter, "severity, open concerns and issues (counts roll up):")?;
        divisions.iter().try_for_each(|division| {
            let placed = Self::placed(&items, &Placement::new(division.clone()));
            writeln!(formatter, "{division}: {}{placed}", self.counts(division, &items))
        })?;
        let root = Self::placed(&items, &Placement::default());
        Some(root).filter(|root| !root.is_empty()).map_or(Ok(()), |root| writeln!(formatter, "{}:{root}", Self::ROOT))
    }
}

#[cfg(test)]
mod tests;
