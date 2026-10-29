use std::fmt;

use crate::distribution::Distribution;
use crate::hundredths::Hundredths;
use crate::leverage::Leverage;
use crate::tally::Tally;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeverageReport {
    measured: Vec<Leverage>,
    without_bodies: Tally,
}

impl LeverageReport {
    #[must_use]
    pub fn new(leverages: Vec<Leverage>) -> LeverageReport {
        let without_bodies =
            Tally::counting(leverages.iter().filter(|leverage| leverage.written() == Tally::default()));
        let mut measured: Vec<Leverage> =
            leverages.into_iter().filter(|leverage| leverage.written() != Tally::default()).collect();
        measured.sort_by(Leverage::lowest_first);
        LeverageReport { measured, without_bodies }
    }

    #[must_use]
    pub fn median(&self) -> Hundredths {
        Distribution::<Hundredths>::sorted(self.measured.iter().map(Leverage::ratio))
            .quarter(Tally::PAIR)
            .copied()
            .unwrap_or_default()
    }
}

impl fmt::Display for LeverageReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "leverage: distinct method names usable on a type / method bodies written for it")?;
        writeln!(
            formatter,
            "  usable = inherent names + the methods of each trait the type has; a local trait contributes its declared methods, a foreign trait the names seen in impls"
        )?;
        writeln!(formatter, "  {}", Distribution::<Hundredths>::sorted(self.measured.iter().map(Leverage::ratio)))?;
        writeln!(formatter, "  types with no bodies: {}", self.without_bodies)?;
        writeln!(formatter, "  lowest:")?;
        self.measured.iter().take(Tally::SHOWN).try_for_each(|leverage| writeln!(formatter, "    {leverage}"))
    }
}
