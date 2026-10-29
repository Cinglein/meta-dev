use std::fmt;

use crate::tally::Tally;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Distribution<Value>(Vec<Value>);

impl<Value: Ord + Clone> Distribution<Value> {
    #[must_use]
    pub fn sorted(values: impl IntoIterator<Item = Value>) -> Distribution<Value> {
        let mut sorted: Vec<Value> = values.into_iter().collect();
        sorted.sort();
        Distribution(sorted)
    }

    #[must_use]
    pub fn quarter(&self, quarters: Tally) -> Option<&Value> {
        Tally::counting(&self.0).quarter_index(quarters).map_or_else(|| self.0.first(), |index| self.0.get(index))
    }
}

impl<Value: Ord + Clone + fmt::Display> fmt::Display for Distribution<Value> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let labels = ["min", "q1", "median", "q3", "max"];
        let quartiles: Vec<String> = Tally::quarters()
            .zip(labels)
            .filter_map(|(quarters, label)| self.quarter(quarters).map(|value| format!("{label} {value}")))
            .collect();
        write!(formatter, "n {}: {}", self.0.len(), quartiles.join(", "))
    }
}
