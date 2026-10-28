use std::io;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Retention {
    #[serde(rename = "files")]
    files_len: usize,
    bytes: usize,
}

impl Retention {
    const FILES_LEN: usize = 200;
    const BYTES: usize = 4_000_000;

    pub fn pruned(self, newest_first: &[PathBuf]) -> io::Result<()> {
        let sized = newest_first.iter().map(|file| (file, std::fs::metadata(file).map_or(0, |meta| meta.len())));
        let totals = sized.scan(0, |total: &mut u64, (file, bytes)| {
            *total = total.saturating_add(bytes);
            Some((file, *total))
        });
        let bytes = u64::try_from(self.bytes).unwrap_or(u64::MAX);
        let dropped = totals.enumerate().filter(|(index, (_, total))| *index >= self.files_len || *total > bytes);
        let removed = dropped.map(|(_, (file, _))| {
            std::fs::remove_file(file).or_else(|failure| file.exists().then_some(failure).map_or(Ok(()), Err))
        });
        removed.collect()
    }
}

impl Default for Retention {
    fn default() -> Self {
        Self { files_len: Self::FILES_LEN, bytes: Self::BYTES }
    }
}
