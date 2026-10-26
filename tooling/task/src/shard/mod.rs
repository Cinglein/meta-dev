mod slice;
mod slice_count;

pub use slice::Slice;
pub use slice_count::SliceCount;

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use crate::command_line::{CargoArgument, CommandLine, MutantsArgument};
use crate::crate_name::CrateName;
use crate::failure::Failure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shard {
    Whole(BTreeSet<CrateName>),
    Slice { crate_name: CrateName, slice: Slice },
}

impl Shard {
    const JOIN: char = '+';
    const CUT: char = '/';

    #[must_use]
    pub fn crates(&self) -> BTreeSet<CrateName> {
        match self {
            Shard::Whole(crates) => crates.clone(),
            Shard::Slice { crate_name, .. } => BTreeSet::from([crate_name.clone()]),
        }
    }

    #[must_use]
    pub fn runs(&self) -> Vec<Shard> {
        match self {
            Shard::Whole(crates) => crates
                .iter()
                .map(|name| Shard::Whole(BTreeSet::from([name.clone()])))
                .collect(),
            Shard::Slice { .. } => vec![self.clone()],
        }
    }

    #[must_use]
    pub fn applied_to(&self, line: CommandLine) -> CommandLine {
        let packaged = self.crates().into_iter().fold(line, |line, name| {
            line.argument(CargoArgument::Package).argument(name)
        });
        match self {
            Shard::Whole(_) => packaged,
            Shard::Slice { slice, .. } => packaged.argument(MutantsArgument::Shard).argument(slice),
        }
    }
}

impl fmt::Display for Shard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shard::Whole(crates) => {
                let names: Vec<String> = crates.iter().map(ToString::to_string).collect();
                formatter.write_str(&names.join(&Self::JOIN.to_string()))
            }
            Shard::Slice { crate_name, slice } => {
                write!(formatter, "{crate_name}{}{slice}", Self::CUT)
            }
        }
    }
}

impl FromStr for Shard {
    type Err = Failure;

    fn from_str(text: &str) -> Result<Shard, Failure> {
        let malformed = || {
            Failure::Usage(format!(
                "{text} is not a shard; expected a+b+c or crate/index/count"
            ))
        };
        match text.split_once(Self::CUT) {
            Some((crate_name, slice)) => {
                let Ok(crate_name) = crate_name.parse::<CrateName>();
                Ok(Shard::Slice {
                    crate_name,
                    slice: slice.parse().map_err(|_| malformed())?,
                })
            }
            None => Ok(Shard::Whole(
                text.split(Self::JOIN)
                    .filter(|name| !name.is_empty())
                    .filter_map(|name| name.parse().ok())
                    .collect(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandLine, Shard};
    use crate::command_line::Program;
    const SLICE: &str = "board/0/4";
    const WHOLE: &str = "eval+uci";
    const SLICE_LINE: &str = "gh --package board --shard 0/4";
    const WHOLE_LINE: &str = "gh --package eval --package uci";

    #[test]
    fn a_shard_prints_back_to_its_spelling_and_names_its_packages_and_slice() {
        let slice: Shard = SLICE.parse().unwrap();
        let whole: Shard = WHOLE.parse().unwrap();
        let line = || CommandLine::new(Program::GitHub);
        assert!(
            slice.to_string() == SLICE
                && whole.to_string() == WHOLE
                && "a/b".parse::<Shard>().is_err()
        );
        assert!(
            slice.applied_to(line()).to_string() == SLICE_LINE
                && whole.applied_to(line()).to_string() == WHOLE_LINE
        );
        assert_eq!((whole.runs().len(), slice.runs().len()), (2, 1));
    }
}
