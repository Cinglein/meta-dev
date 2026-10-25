mod tally;

use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use crate::baseline::Baseline;
use crate::drift::Drift;
use crate::failure::Failure;
use crate::lint_config::LintConfig;
use crate::prior::Prior;
use crate::ratchet::Ratchet;
use crate::rule::{CrateGraph, Findings, GraphReport};
use crate::site::Site;
use crate::source_file::SourceFile;
use crate::violation_count::ViolationCount;
use tally::Tally;

#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
    config: LintConfig,
}

impl Workspace {
    const TARGET: &str = "target";

    pub fn new(root: PathBuf, config: LintConfig) -> Workspace {
        Workspace { root, config }
    }

    pub fn lint(&self, prior: &Prior) -> Result<Tally, Failure> {
        let files = self.source_files()?;
        let allowed = Baseline::read(&self.baseline_path())?.unwrap_or_default();
        prior.admit(&allowed)?;
        let findings = Findings::collect(&files, &self.config);
        let observed = findings.observed();
        let drifts = observed.drifts(&allowed);
        if drifts.is_empty() {
            Ok(Tally::new(ViolationCount::of_len(files.len()), observed))
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
            Ok(Tally::new(ViolationCount::of_len(files.len()), observed))
        } else {
            Err(Failure::Ratchet(Box::new(Ratchet::new(grown, findings, observed, allowed.unwrap_or_default()))))
        }
    }

    pub fn crate_graphs(&self) -> Result<GraphReport, Failure> {
        Ok(CrateGraph::graph_report(&self.source_files()?))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn baseline_path(&self) -> PathBuf {
        self.root.join(self.config.baseline())
    }

    fn source_files(&self) -> Result<Vec<SourceFile>, Failure> {
        let mut files = self.rust_files_under(&self.root)?;
        files.sort();
        files
            .iter()
            .map(|file| SourceFile::read(self, file))
            .collect::<Result<Vec<SourceFile>, Failure>>()
            .and_then(|files| if files.is_empty() { Err(Failure::NoFiles) } else { Ok(files) })
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
        } else if path == self.root.join(Self::TARGET) || self.config.excludes(&entry.file_name()) {
            Ok(Vec::new())
        } else {
            self.rust_files_under(&path)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::failure::Failure;
    use crate::fixture::Fixture;

    const CONFIG: &str = "xtask.toml";
    const CONFIG_TEXT: &str = "surfaces = []\n[lints]\nbaseline = \"lint-baseline.toml\"\nexclude = [\".git\"]\n[lints.crate_graph]\nmax_scc = 1\n";
    const BASELINE: &str = "lint-baseline.toml";
    const BASELINE_TEXT: &str = "[crate-graph]\n\"crates/a\" = 1\n";
    const RAISED_BASELINE: &str = "[crate-graph]\n\"crates/a\" = 2\n";
    const TANGLED: &str = "crates/a/src/lib.rs";
    const TANGLED_TEXT: &str = "pub struct Alpha(Box<Beta>);\npub struct Beta(Box<Alpha>);\n";
    const MORE_TANGLED: &str =
        "pub struct Alpha(Box<Beta>);\npub struct Beta(Box<Gamma>);\npub struct Gamma(Box<Alpha>);\n";
    const CLEAN: &str = "crates/b/src/lib.rs";
    const MOVED: &str = "crates/c/src/lib.rs";
    const SOURCES: &str = "crates";

    impl Fixture {
        fn tangled() -> Fixture {
            let fixture = Fixture::new();
            fixture
                .write(CONFIG, CONFIG_TEXT)
                .write(BASELINE, BASELINE_TEXT)
                .write(TANGLED, TANGLED_TEXT)
                .write(CLEAN, "pub struct Delta;\n")
                .branch();
            fixture
        }

        fn ratchet(&self) -> Result<(), Failure> {
            let project = self.project();
            project.prior().and_then(|prior| project.lints().lint(&prior)).map(|_| ())
        }
    }

    #[test]
    fn a_raised_baseline_is_refused_and_a_deleted_one_fails_the_ratchet() {
        let fixture = Fixture::tangled();
        fixture.write(TANGLED, MORE_TANGLED).write(BASELINE, RAISED_BASELINE);
        assert!(matches!(fixture.ratchet(), Err(Failure::Raised(_))));
        fs::remove_file(fixture.path(BASELINE)).unwrap();
        assert!(matches!(fixture.ratchet(), Err(Failure::Ratchet(_))));
    }

    #[test]
    fn no_files_fails() {
        let fixture = Fixture::tangled();
        fs::remove_dir_all(fixture.path(SOURCES)).unwrap();
        assert!(matches!(fixture.ratchet(), Err(Failure::NoFiles)));
    }

    #[test]
    fn a_moved_crate_with_the_same_violations_passes() {
        let fixture = Fixture::tangled();
        fixture.write(MOVED, TANGLED_TEXT);
        fs::remove_file(fixture.path(TANGLED)).unwrap();
        assert!(fixture.ratchet().is_ok());
    }
}
