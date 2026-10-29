use std::collections::BTreeSet;
use std::path::PathBuf;

use task::{FailureKind, FailureSet};

use super::cargo_line::CargoLine;
use super::message::Message;

#[derive(Debug)]
pub(super) struct Diagnostics {
    errors: Vec<Message>,
}

impl Diagnostics {
    pub(super) fn parse(printed: &str) -> Diagnostics {
        let errors = printed
            .lines()
            .filter_map(|line| serde_json::from_str::<CargoLine>(line).ok())
            .filter_map(CargoLine::message)
            .filter(Message::is_error)
            .collect();
        Diagnostics { errors }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub(super) fn found(&self) -> FailureSet {
        let paths = |lint: bool| -> BTreeSet<PathBuf> {
            self.errors.iter().filter(|message| message.is_lint() == lint).flat_map(Message::primary_paths).collect()
        };
        let [compile, lint] = [false, true].map(paths);
        let kind = |paths: BTreeSet<PathBuf>, kind: fn(Vec<PathBuf>) -> FailureKind| {
            (!paths.is_empty()).then(|| kind(paths.into_iter().collect()))
        };
        [kind(compile, |paths| FailureKind::Compile { paths }), kind(lint, |paths| FailureKind::Lint { paths })]
            .into_iter()
            .flatten()
            .fold(FailureSet::default(), FailureSet::with)
    }

    pub(super) fn rendered(&self) -> String {
        self.errors.iter().filter_map(Message::rendered).collect::<Vec<&str>>().join("\n")
    }
}
