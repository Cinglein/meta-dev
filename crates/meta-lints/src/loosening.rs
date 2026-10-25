use std::fmt;
use std::path::PathBuf;

use crate::drift::Drift;
use crate::path_prefix::PathPrefix;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Loosening {
    BaselineMoved { from: PathBuf, to: PathBuf },
    Excluded(PathBuf),
    ProductDropped(PathPrefix),
    BoundaryAdded(PathPrefix),
    Raised(Drift),
}

impl fmt::Display for Loosening {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Loosening::BaselineMoved { from, to } => {
                write!(formatter, "baseline moved from {} to {}", from.display(), to.display())
            }
            Loosening::Excluded(directory) => write!(formatter, "exclude widened: {}", directory.display()),
            Loosening::ProductDropped(prefix) => write!(formatter, "product narrowed: {prefix}"),
            Loosening::BoundaryAdded(prefix) => write!(formatter, "boundary widened: {prefix}"),
            Loosening::Raised(drift) => write!(formatter, "baseline raised above the merge base: {drift}"),
        }
    }
}
