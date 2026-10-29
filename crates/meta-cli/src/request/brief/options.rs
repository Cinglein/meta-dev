use meta_schema::{Actor, Text};

use crate::request::repository::Repository;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BriefOptions {
    actor: Actor,
    scratch: Option<Text>,
    repository: Repository,
}

impl BriefOptions {
    const HERE: &'static str = ".";

    #[must_use]
    pub fn new(actor: Actor, scratch: Option<Text>, repository: Repository) -> Self {
        Self { actor, scratch, repository }
    }

    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    pub fn repository(&self) -> &Repository {
        &self.repository
    }

    pub fn scratch(&self) -> String {
        self.scratch.as_ref().map_or_else(|| Self::HERE.to_owned(), ToString::to_string)
    }
}
