use std::fmt;

use serde::{Deserialize, Serialize};

use crate::acceptance_failure::AcceptanceFailure;
use crate::acceptance_row::AcceptanceRow;
use crate::acceptance_rows::AcceptanceRows;
use crate::span_name::SpanName;
use crate::vec1::Vec1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observability {
    rows: AcceptanceRows,
    spans: Vec1<SpanName>,
}

impl Observability {
    pub fn named(rows: Vec<AcceptanceRow>, spans: Vec<SpanName>) -> Result<Self, AcceptanceFailure> {
        Ok(Self { rows: AcceptanceRows::try_from(rows)?, spans: Vec1::try_from(spans)? })
    }
}

impl fmt::Display for Observability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rows = self.rows.rows().as_slice().iter().map(ToString::to_string);
        let spans = self.spans.as_slice().iter().map(|span| format!("span {span}"));
        formatter.write_str(&rows.chain(spans).collect::<Vec<_>>().join(" "))
    }
}
