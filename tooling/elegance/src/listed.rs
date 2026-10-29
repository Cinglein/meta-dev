use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Listed<Item>(BTreeSet<Item>);

impl<Item: Ord> Listed<Item> {
    #[must_use]
    pub fn new(items: impl IntoIterator<Item = Item>) -> Listed<Item> {
        Listed(items.into_iter().collect())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<Item: fmt::Display> fmt::Display for Listed<Item> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        write!(formatter, "{}", items.join(", "))
    }
}
