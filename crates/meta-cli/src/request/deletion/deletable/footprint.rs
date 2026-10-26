use std::path::Path;

use meta_schema::ByteCount;

#[derive(Debug)]
pub struct Footprint;

impl Footprint {
    pub fn measured(path: &Path) -> ByteCount {
        match path.symlink_metadata() {
            Ok(entry) if entry.is_dir() => {
                std::fs::read_dir(path).into_iter().flatten().flatten().map(|child| Self::measured(&child.path())).sum()
            }
            Ok(entry) => ByteCount::new(entry.len()),
            Err(_) => ByteCount::default(),
        }
    }
}
