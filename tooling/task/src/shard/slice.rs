use std::fmt;
use std::str::FromStr;

use super::slice_count::SliceCount;
use crate::failure::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slice {
    index: usize,
    count: SliceCount,
}

impl Slice {
    const CUT: char = '/';

    pub fn all(count: SliceCount) -> impl Iterator<Item = Slice> {
        count.index_range().map(move |index| Slice { index, count })
    }
}

impl fmt::Display for Slice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}{}", self.index, Self::CUT, self.count)
    }
}

impl FromStr for Slice {
    type Err = Failure;

    fn from_str(text: &str) -> Result<Slice, Failure> {
        let malformed = || Failure::Usage(format!("{text} is not a slice; expected index/count"));
        let (index, count) = text.split_once(Self::CUT).ok_or_else(malformed)?;
        let count: SliceCount = count.parse().map_err(|_| malformed())?;
        let index: usize = index.parse().map_err(|_| malformed())?;
        count
            .index_range()
            .contains(&index)
            .then_some(Slice { index, count })
            .ok_or_else(malformed)
    }
}

#[cfg(test)]
mod tests {
    use super::{Slice, SliceCount};
    use std::num::NonZeroUsize;
    const SPELLED: [&str; 2] = ["0/2", "1/2"];
    const REJECTED: [&str; 4] = ["2/2", "0/0", "1", "a/2"];

    #[test]
    fn every_slice_of_a_count_spells_back_and_an_index_past_the_count_or_a_zero_count_is_rejected()
    {
        let all: Vec<String> = NonZeroUsize::new(SPELLED.len())
            .map(SliceCount::from)
            .into_iter()
            .flat_map(Slice::all)
            .map(|slice| slice.to_string())
            .collect();
        assert!(all == SPELLED && SPELLED.iter().all(|text| text.parse::<Slice>().is_ok()));
        assert!(REJECTED.iter().all(|text| text.parse::<Slice>().is_err()));
    }
}
