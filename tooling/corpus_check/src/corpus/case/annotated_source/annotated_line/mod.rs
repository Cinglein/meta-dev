mod annotation;

use std::fmt;
use std::path::Path;
use std::str::FromStr;

use annotation::Annotation;

use super::super::super::corpus_error::CorpusError;
use super::super::super::expectation::Expectation;
use task::LineNumber;
use task::Name;

pub struct AnnotatedLine {
    code: String,
    annotations: Vec<Annotation>,
}

impl AnnotatedLine {
    const MARK: &str = "//~";

    pub fn expectations(&self, case: &Name, path: &Path, line: LineNumber) -> Vec<Expectation> {
        self.annotations.iter().map(|annotation| annotation.placed(case, path, line)).collect()
    }
}

impl FromStr for AnnotatedLine {
    type Err = CorpusError;

    fn from_str(text: &str) -> Result<AnnotatedLine, CorpusError> {
        Ok(AnnotatedLine {
            code: text.split(Self::MARK).next().unwrap_or_default().trim_end().to_owned(),
            annotations: text
                .split(Self::MARK)
                .skip(1)
                .map(str::parse)
                .collect::<Result<Vec<Annotation>, CorpusError>>()?,
        })
    }
}

impl fmt::Display for AnnotatedLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.code)
    }
}
