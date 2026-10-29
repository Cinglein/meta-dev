use std::io;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Retention {
    #[serde(rename = "traces")]
    traces_len: usize,
    bytes: usize,
}

impl Retention {
    const TRACES_LEN: usize = 200;
    const BYTES: usize = 4_000_000;

    pub fn pruned(self, newest_first: &[Vec<PathBuf>]) -> io::Result<()> {
        let size = |file| std::fs::metadata(file).map_or(0, |meta| meta.len());
        let sized = newest_first.iter().map(|files| (files, files.iter().map(size).sum::<u64>()));
        let totals = sized.scan(0, |total: &mut u64, (files, bytes)| {
            *total = total.saturating_add(bytes);
            Some((files, *total))
        });
        let bytes = u64::try_from(self.bytes).unwrap_or(u64::MAX);
        let dropped = totals.enumerate().filter(|(index, (_, total))| *index >= self.traces_len || *total > bytes);
        let removed = dropped.flat_map(|(_, (files, _))| files).map(|file| {
            std::fs::remove_file(file).or_else(|failure| file.exists().then_some(failure).map_or(Ok(()), Err))
        });
        removed.collect()
    }
}

impl Default for Retention {
    fn default() -> Self {
        Self { traces_len: Self::TRACES_LEN, bytes: Self::BYTES }
    }
}
