use std::fmt;

use syn::LitInt;

use super::super::super::amount::Amount;

#[derive(Clone)]
pub(super) struct LargeLiteral {
    literal: LitInt,
    limit: Amount,
}

impl LargeLiteral {
    pub(super) fn new(literal: LitInt, limit: Amount) -> LargeLiteral {
        LargeLiteral { literal, limit }
    }
}

impl fmt::Display for LargeLiteral {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "integer literal {} in test code, at most {} allowed", self.literal, self.limit)
    }
}
