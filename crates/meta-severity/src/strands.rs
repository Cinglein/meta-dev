use std::fmt;

use crate::stranded::Stranded;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Strands(Vec<Stranded>);

impl Strands {
    #[must_use]
    pub fn items(&self) -> &[Stranded] {
        &self.0
    }
}

impl FromIterator<Stranded> for Strands {
    fn from_iter<Items: IntoIterator<Item = Stranded>>(items: Items) -> Strands {
        Strands(items.into_iter().collect())
    }
}

impl fmt::Display for Strands {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|stranded| writeln!(formatter, "{stranded}"))?;
        self.0.first().map_or(Ok(()), |_| {
            write!(formatter, "rate each on its division's scale, or change the scales in tooling/policy.toml")
        })
    }
}
