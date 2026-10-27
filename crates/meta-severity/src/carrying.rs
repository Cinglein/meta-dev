use std::cmp::Ordering;

use meta_schema::{Carry, Change, Division, LevelName, Rating, Scenario};

use crate::rating_failure::RatingFailure;
use crate::scales::Scales;

#[derive(Clone, Copy, Debug)]
pub struct Carrying<'policy> {
    scales: &'policy Scales,
    from: Option<&'policy Division>,
    to: &'policy Division,
}

impl<'policy> Carrying<'policy> {
    #[must_use]
    pub fn new(scales: &'policy Scales, from: Option<&'policy Division>, to: &'policy Division) -> Carrying<'policy> {
        Carrying { scales, from, to }
    }

    pub fn carried(
        &self,
        current: &Rating,
        level: Option<LevelName>,
        scenario: Option<Scenario>,
    ) -> Result<Carry, RatingFailure> {
        let scale = self.scales.scale(self.to).ok_or(RatingFailure::Unscaled)?;
        let known =
            |level: LevelName| scale.level(&level).map(|_| level.clone()).ok_or(RatingFailure::UnknownLevel(level));
        match (current.level().cloned(), level) {
            (None, None) => Ok(Carry::Unrated),
            (None, Some(level)) => Ok(Carry::Changed { change: Change::First { level: known(level)? } }),
            (Some(from), None) => {
                scale.level(&from).ok_or_else(|| RatingFailure::Unplaced(from.clone(), self.to.clone()))?;
                (self.severity(&from, &from) == Some(Ordering::Equal))
                    .then(|| Carry::Kept { from: from.clone(), level: from.clone() })
                    .ok_or_else(|| RatingFailure::Remapped(from.clone(), self.to.clone()))
            }
            (Some(from), Some(level)) => self.changed(from, known(level)?, scenario),
        }
    }

    fn changed(&self, from: LevelName, level: LevelName, scenario: Option<Scenario>) -> Result<Carry, RatingFailure> {
        match self.severity(&from, &level) {
            Some(Ordering::Equal) => Ok(Carry::Kept { from, level }),
            Some(Ordering::Less) => Ok(Carry::Changed { change: Change::Lowered { from, level } }),
            Some(Ordering::Greater) | None => scenario
                .map(|scenario| Carry::Changed { change: Change::Raised { from: from.clone(), level, scenario } })
                .ok_or(RatingFailure::RaisedWithoutScenario(from)),
        }
    }

    fn severity(&self, old: &LevelName, new: &LevelName) -> Option<Ordering> {
        let from = self.from?;
        let [old_lineage, new_lineage] = [from.lineage(), self.to.lineage()];
        let common: Vec<(Division, Division)> = old_lineage
            .iter()
            .filter(|above| new_lineage.contains(above))
            .map(|above| (above.clone(), above.clone()))
            .collect();
        let tops = old_lineage.first().cloned().zip(new_lineage.first().cloned());
        let pairs = if common.is_empty() { tops.into_iter().collect() } else { common };
        let compared = pairs.iter().map(|(old_at, new_at)| {
            let [old_scale, new_scale] = [old_at, new_at].map(|at| self.scales.scale(at));
            old_scale?.same_levels(new_scale?).then_some(())?;
            let old_rank = old_scale?.index_of(&self.scales.rolled_up(from, old, old_at)?)?;
            let new_rank = new_scale?.index_of(&self.scales.rolled_up(self.to, new, new_at)?)?;
            Some(old_rank.cmp(&new_rank))
        });
        compared.collect::<Option<Vec<Ordering>>>()?.into_iter().reduce(Self::stricter)
    }

    fn stricter(first: Ordering, second: Ordering) -> Ordering {
        match (first, second) {
            (Ordering::Greater, _) | (_, Ordering::Greater) => Ordering::Greater,
            (Ordering::Less, _) | (_, Ordering::Less) => Ordering::Less,
            (Ordering::Equal, Ordering::Equal) => Ordering::Equal,
        }
    }
}
