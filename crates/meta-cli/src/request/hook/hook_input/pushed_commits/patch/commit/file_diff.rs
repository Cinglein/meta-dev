mod hunk;

use std::convert::Infallible;
use std::str::FromStr;

use crate::request::hook::hook_input::pushed_commits::patch::Leak;
use hunk::Hunk;

#[derive(Debug)]
pub struct FileDiff {
    name: String,
    hunks: String,
}

impl FileDiff {
    const HUNK_START: &'static str = "\n@@";
    const NEW_FILE: &'static str = "+++ ";
    const DESTINATION: &'static str = "b/";
    const QUOTE: char = '"';

    pub fn first_leak(&self, commit: &str) -> Option<Leak> {
        self.hunks.split(Self::HUNK_START).find_map(|text| {
            let Ok(hunk) = text.parse::<Hunk>();
            hunk.first_scratch_line().map(|line| Leak::new(commit, &self.name, line))
        })
    }
}

impl FromStr for FileDiff {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (header, hunks) = text.split_once(Self::HUNK_START).unwrap_or((text, ""));
        let named = header.lines().find_map(|line| line.strip_prefix(Self::NEW_FILE)).unwrap_or_default();
        let bare = named.trim_start_matches(Self::QUOTE);
        Ok(Self { name: bare.strip_prefix(Self::DESTINATION).unwrap_or(bare).to_owned(), hunks: hunks.to_owned() })
    }
}
