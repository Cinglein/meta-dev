use std::fs; use std::{fs::File, io}; //~ primitive-boundary: fs reaches //~ primitive-boundary: fs reaches
use std::{io::Write, os::unix::fs::symlink}; use tokio::fs as files; //~ primitive-boundary: fs reaches //~ primitive-boundary: fs reaches
pub struct Files; impl Files { pub fn head(dir: &std::path::Path) -> Option<std::path::PathBuf> { std::fs::canonicalize(dir.join("HEAD")).ok() } } //~ primitive-boundary: fs reaches
impl Files { pub fn held(name: &str) -> bool { File::open(name).is_ok() && File::options().open(name).is_ok() && ::std::fs::metadata(name).is_ok() } } //~ primitive-boundary: open reaches //~ primitive-boundary: options reaches //~ primitive-boundary: fs reaches
#[cfg(test)] mod tests { #[test] fn laid() { assert!(std::fs::write(std::path::Path::new(".git/MERGE_HEAD"), "").is_err()); } } //~ primitive-boundary: fs reaches
impl Files { pub fn fetched(fs: &str, refs: &str) -> bool { fs.is_empty() && refs.is_empty() && "FETCH_HEAD".is_empty() } }
