mod annotated_line;

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use annotated_line::AnnotatedLine;

use super::super::corpus_error::CorpusError;
use super::super::expectation::Expectation;
use task::Failure;
use task::LineNumber;
use task::Name;
use task::SourceFile;

pub struct AnnotatedSource {
    lines: Vec<AnnotatedLine>,
}

impl AnnotatedSource {
    pub fn expectations(&self, case: &Name, path: &Path) -> Vec<Expectation> {
        self.lines
            .iter()
            .zip(1..)
            .flat_map(|(line, number)| line.expectations(case, path, LineNumber::new(number)))
            .collect()
    }

    pub fn source_file(&self, path: PathBuf) -> Result<SourceFile, Failure> {
        SourceFile::parse(path, self.to_string())
    }
}

impl FromStr for AnnotatedSource {
    type Err = CorpusError;

    fn from_str(text: &str) -> Result<AnnotatedSource, CorpusError> {
        Ok(AnnotatedSource {
            lines: text.lines().map(str::parse).collect::<Result<Vec<AnnotatedLine>, CorpusError>>()?,
        })
    }
}

impl fmt::Display for AnnotatedSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.lines.iter().try_for_each(|line| writeln!(formatter, "{line}"))
    }
}
