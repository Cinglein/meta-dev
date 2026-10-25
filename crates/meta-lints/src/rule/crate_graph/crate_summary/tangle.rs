use std::fmt;

use crate::rule::crate_graph::item_count::ItemCount;
use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tangle(Vec<Name>);

impl Tangle {
    pub fn new(names: Vec<Name>) -> Tangle {
        Tangle(names)
    }

    pub fn items(&self) -> ItemCount {
        ItemCount::of_len(self.0.len())
    }
}

impl fmt::Display for Tangle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        write!(formatter, "cycle of {} items: {}", self.0.len(), names.join(" "))
    }
}
