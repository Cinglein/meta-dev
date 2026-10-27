use std::fmt;

use crate::demotion::Demotion;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Demotions(Vec<Demotion>);

impl Demotions {
    #[must_use]
    pub fn items(&self) -> &[Demotion] {
        &self.0
    }
}

impl FromIterator<Demotion> for Demotions {
    fn from_iter<Items: IntoIterator<Item = Demotion>>(items: Items) -> Demotions {
        Demotions(items.into_iter().collect())
    }
}

impl fmt::Display for Demotions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|demotion| writeln!(formatter, "{demotion}"))
    }
}
