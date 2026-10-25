mod tally;

use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use crate::baseline::Baseline;
use crate::count::Count;
use crate::drift::Drift;
use crate::failure::Failure;
use crate::lint_config::LintConfig;
use crate::ratchet::Ratchet;
use crate::rule::Findings;
use crate::site::Site;
use crate::source_file::SourceFile;
use tally::Tally;

#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
    config: LintConfig,
}

impl Workspace {
    pub fn new(root: PathBuf, config: LintConfig) -> Workspace {
        Workspace { root, config }
    }

    pub fn lint(&self) -> Result<Tally, Failure> {
        let files = self.source_files()?;
        let findings = Findings::collect(&files, &self.config);
        let observed = findings.observed();
        let allowed = Baseline::read(&self.baseline_path())?.unwrap_or_default();
        let drifts = observed.drifts(&allowed);
        if drifts.is_empty() {
            Ok(Tally::new(Count::of_len(files.len()), observed))
        } else {
            Err(Failure::Ratchet(Box::new(Ratchet::new(drifts, findings, observed, allowed))))
        }
    }

    pub fn shrink_baseline(&self) -> Result<Tally, Failure> {
        let files = self.source_files()?;
        let findings = Findings::collect(&files, &self.config);
        let observed = findings.observed();
        let allowed = Baseline::read(&self.baseline_path())?;
        let grown: Vec<Drift> =
            allowed.iter().flat_map(|allowed| observed.drifts(allowed)).filter(Drift::grew).collect();
        if grown.is_empty() {
            observed.write(&self.baseline_path())?;
            Ok(Tally::new(Count::of_len(files.len()), observed))
        } else {
            Err(Failure::Ratchet(Box::new(Ratchet::new(grown, findings, observed, allowed.unwrap_or_default()))))
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config(&self) -> &LintConfig {
        &self.config
    }

    fn baseline_path(&self) -> PathBuf {
        self.root.join(self.config.baseline())
    }

    fn source_files(&self) -> Result<Vec<SourceFile>, Failure> {
        let mut files = self.rust_files_under(&self.root)?;
        files.sort();
        files.iter().map(|file| SourceFile::read(self, file)).collect()
    }

    fn rust_files_under(&self, directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        let io = |error| Failure::Io { site: Site::of_path(directory), error };
        fs::read_dir(directory)
            .map_err(io)?
            .map(|entry| entry.map_err(io).and_then(|entry| self.rust_files_in(&entry)))
            .collect::<Result<Vec<Vec<PathBuf>>, Failure>>()
            .map(|nested| nested.into_iter().flatten().collect())
    }

    fn rust_files_in(&self, entry: &DirEntry) -> Result<Vec<PathBuf>, Failure> {
        let path = entry.path();
        if !path.is_dir() {
            Ok(path.extension().is_some_and(|extension| extension == "rs").then_some(path).into_iter().collect())
        } else if self.config.excludes(&entry.file_name()) {
            Ok(Vec::new())
        } else {
            self.rust_files_under(&path)
        }
    }
}
