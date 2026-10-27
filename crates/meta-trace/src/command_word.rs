use std::fmt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::trace_file_name::TraceFileName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandWord(String);

impl CommandWord {
    const SHOWN_LEN: usize = 24;
    const UNNAMED: &'static str = "meta";
    const JOINER: char = '-';

    #[must_use]
    pub fn file_in(&self, dir: &Path, at: DateTime<Utc>) -> PathBuf {
        dir.join(TraceFileName::new(at, self).to_string())
    }
}

impl From<&[String]> for CommandWord {
    fn from(args: &[String]) -> Self {
        let word = args.get(1).map(String::as_str).unwrap_or_default();
        let kept = word.chars().filter(|letter| letter.is_ascii_lowercase() || *letter == Self::JOINER);
        let kept: String = kept.take(Self::SHOWN_LEN).collect();
        Self(Some(kept).filter(|kept| !kept.is_empty()).unwrap_or_else(|| Self::UNNAMED.to_owned()))
    }
}

impl fmt::Display for CommandWord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
