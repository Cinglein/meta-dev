use std::fmt;

use syn::LitInt;

use super::super::test_count::TestCount;
use task::Most;
use task::Name;

pub(super) enum Finding {
    TooManyTests { tests: TestCount, most: Most },
    TestOverrun { name: Name, overrun: String },
    LargeLiteral { literal: LitInt, most: Most },
    AssertionOutsideTest,
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::TooManyTests { tests, most } => {
                write!(formatter, "{tests} tests, at most {most} allowed")
            }
            Finding::TestOverrun { name, overrun } => write!(formatter, "fn {name} {overrun}"),
            Finding::LargeLiteral { literal, most } => write!(
                formatter,
                "integer literal {literal} in test code, at most {most} allowed"
            ),
            Finding::AssertionOutsideTest => formatter
                .write_str("assertion outside a test; helpers return values and tests assert them"),
        }
    }
}
