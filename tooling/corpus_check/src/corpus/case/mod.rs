mod annotated_source;

use std::fs;
use std::path::{Component, Path};

use annotated_source::AnnotatedSource;

use super::corpus_error::CorpusError;
use super::expectation::Expectation;
use crate::lint_name::LintName;
use crate_graph::{CrateGraph, GraphReport};
use task::CrateName;
use task::Name;
use task::Policy;
use task::Site;
use task::SourceFile;
use task::Workspace;

pub struct Case {
    name: Name,
    files: Vec<SourceFile>,
    graphs: GraphReport,
    expected: Vec<Expectation>,
}

impl Case {
    const SOURCE_DIRECTORY: &str = "src";

    pub fn load(directory: &Path) -> Result<Case, CorpusError> {
        let empty = Case {
            name: Name::from(&*directory.file_name().unwrap_or_default().to_string_lossy()),
            files: Vec::new(),
            graphs: GraphReport::default(),
            expected: Vec::new(),
        };
        let mut files = Workspace::rust_files_under(directory)?;
        files.sort();
        let read = files.iter().try_fold(empty, |case, file| case.reading(directory, file))?;
        Ok(Case { graphs: CrateGraph::graphs(&read.files), ..read })
    }

    pub fn expectations(&self) -> &[Expectation] {
        &self.expected
    }

    pub fn findings(&self, lint: LintName, policy: &Policy) -> Vec<Expectation> {
        lint.report(&self.files, &self.graphs, policy)
            .violations()
            .iter()
            .map(|violation| Expectation::found(&self.name, lint, violation))
            .collect()
    }

    pub fn expects(&self, lint: LintName) -> bool {
        self.expected.iter().any(|expected| expected.is_for(lint))
    }

    fn reading(mut self, directory: &Path, file: &Path) -> Result<Case, CorpusError> {
        let path = file.strip_prefix(directory).unwrap_or(file).to_path_buf();
        let text =
            fs::read_to_string(file).map_err(|error| CorpusError::Io { site: Site::File(path.clone()), error })?;
        let source: AnnotatedSource = text.parse()?;
        self.expected.extend(source.expectations(&self.name, &path));
        let parsed = source.source_file(path.clone())?;
        self.files.push(match Self::crate_of(&path) {
            Some(crate_name) => parsed.in_crate(crate_name),
            None => parsed,
        });
        Ok(self)
    }

    fn crate_of(path: &Path) -> Option<CrateName> {
        let components: Vec<Component<'_>> = path.components().collect();
        components
            .iter()
            .zip(components.iter().skip(1))
            .find(|(_, next)| next.as_os_str() == Self::SOURCE_DIRECTORY)
            .and_then(|(owner, _)| owner.as_os_str().to_str())
            .and_then(|name| name.parse().ok())
    }
}
