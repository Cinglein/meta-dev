mod case;
mod corpus_error;
mod expectation;
mod mismatch;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use case::Case;
use corpus_error::CorpusError;
use expectation::Expectation;
use meta_door::Door;
use mismatch::Mismatch;
use strum::IntoEnumIterator;

use crate::lint_name::LintName;
use task::Policy;
use task::Site;

pub struct Corpus {
    cases: Vec<Case>,
}

impl Corpus {
    const DIRECTORY: &str = "../corpus";

    pub fn load() -> Result<Corpus, CorpusError> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(Self::DIRECTORY);
        let io = |error| CorpusError::Io { site: Site::File(root.clone()), error };
        let directories: BTreeSet<PathBuf> = Door::listed(&root).map_err(io)?.into_iter().collect();
        Ok(Corpus {
            cases: directories
                .iter()
                .filter(|directory| Door::kind(directory) == meta_door::kinds::Kind::Dir)
                .map(|directory| Case::load(directory))
                .collect::<Result<Vec<Case>, CorpusError>>()?,
        })
    }

    pub fn mismatch(&self, policy: &Policy) -> Mismatch {
        let expected: Vec<Expectation> = self.cases.iter().flat_map(Case::expectations).cloned().collect();
        let found: Vec<Expectation> = self
            .cases
            .iter()
            .flat_map(|case| LintName::iter().flat_map(move |lint| case.findings(lint, policy)))
            .collect();
        Mismatch::between(expected, found)
    }

    pub fn expects(&self, lint: LintName) -> bool {
        self.cases.iter().any(|case| case.expects(lint))
    }
}
