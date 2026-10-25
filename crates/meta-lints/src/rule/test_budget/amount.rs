use std::fmt;
use std::num::ParseIntError;
use std::ops::Add;
use std::str::FromStr;

use syn::spanned::Spanned;
use syn::{Block, ItemMod};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Amount(usize);

impl Amount {
    pub(super) const ONE: Amount = Amount(1);
    const PERCENT: usize = 100;

    pub(super) const fn new(amount: usize) -> Amount {
        Amount(amount)
    }

    pub(super) fn percent(self, share: Amount) -> Amount {
        Amount(self.0 * share.0 / Self::PERCENT)
    }

    pub(super) fn lines_spanned(module: &ItemMod) -> Amount {
        let span = module.span();
        Amount(span.end().line - span.start().line + 1)
    }

    pub(super) fn lines_inside(block: &Block) -> Amount {
        let span = block.span();
        Amount((span.end().line - span.start().line).saturating_sub(1))
    }
}

impl From<bool> for Amount {
    fn from(present: bool) -> Amount {
        Amount(usize::from(present))
    }
}

impl Add for Amount {
    type Output = Amount;

    fn add(self, other: Amount) -> Amount {
        Amount(self.0 + other.0)
    }
}

impl FromStr for Amount {
    type Err = ParseIntError;

    fn from_str(text: &str) -> Result<Amount, ParseIntError> {
        text.parse().map(Amount)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
