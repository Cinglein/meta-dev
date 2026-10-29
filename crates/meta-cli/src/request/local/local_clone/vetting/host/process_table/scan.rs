use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum Scan {
    Opened(Vec<PathBuf>),
    Skipped,
    #[cfg_attr(not(any(target_os = "linux", test)), expect(dead_code, reason = "only procfs counts these"))]
    Uninspected,
}
