use std::fs; use std::{fs::File, io}; use std::fs::{DirEntry, Permissions}; use std::os::unix::fs::PermissionsExt; //~ primitive-boundary: fs reaches //~ primitive-boundary: fs reaches //~ primitive-boundary: fs reaches //~ primitive-boundary: fs reaches
use std::{io::Write, os::unix::fs::symlink}; use tokio::fs as files; use std::fs::{self, ReadDir}; use std::fs::OpenOptions; //~ primitive-boundary: fs reaches //~ primitive-boundary: fs reaches //~ primitive-boundary: fs reaches //~ primitive-boundary: fs reaches
pub struct Files; impl Files { pub fn head(dir: &std::path::Path) -> Option<std::path::PathBuf> { std::fs::canonicalize(dir.join("HEAD")).ok() } } //~ primitive-boundary: fs reaches
impl Files { pub fn held(name: &str) -> bool { File::open(name).is_ok() && File::options().open(name).is_ok() && ::std::fs::metadata(name).is_ok() } } //~ primitive-boundary: open reaches //~ primitive-boundary: options reaches //~ primitive-boundary: fs reaches
#[cfg(test)] mod tests { #[test] fn laid() { assert!(std::fs::write(std::path::Path::new(".git/MERGE_HEAD"), "").is_err()); } } //~ primitive-boundary: fs reaches
impl Files { pub fn fetched(fs: &str, refs: &str) -> bool { fs.is_empty() && refs.is_empty() && "FETCH_HEAD".is_empty() } }
impl Files { pub fn tricks(std: &fs) -> bool { std >= fs && other::fs::write() && Dir::create(std) } }
use other::{fs, io}; use tokio::{fs, io as tio}; //~ primitive-boundary: fs reaches
m!(std::[fs]); impl Files { pub fn compared() -> bool { std::fs <= X } } //~ primitive-boundary: fs reaches
impl Files { pub fn made(name: &std::path::Path) -> bool { File::create_new(name).is_ok() && std::fs::DirBuilder::new().create(name).is_ok() } } //~ primitive-boundary: create_new reaches //~ primitive-boundary: fs reaches
