use std::collections::BTreeMap;
use std::fmt;

use crate::micros::Micros;
use crate::span_kind::SpanKind;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct KindTimes(BTreeMap<SpanKind, Micros>);

impl FromIterator<(SpanKind, Micros)> for KindTimes {
    fn from_iter<Times: IntoIterator<Item = (SpanKind, Micros)>>(times: Times) -> Self {
        let mut totals = BTreeMap::new();
        times.into_iter().for_each(|(kind, time)| {
            let total: &mut Micros = totals.entry(kind).or_default();
            *total = *total + time;
        });
        Self(totals)
    }
}

impl fmt::Display for KindTimes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ranked: Vec<_> = self.0.iter().filter(|(_, time)| **time > Micros::default()).collect();
        ranked.sort_by(|left, right| right.1.cmp(left.1).then(left.0.cmp(right.0)));
        ranked.iter().try_for_each(|(kind, time)| writeln!(formatter, "  {time:>9} {kind}"))
    }
}
