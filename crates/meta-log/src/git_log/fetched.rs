use super::content::Content;
use super::sha::Sha;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fetched {
    tip: Option<Sha>,
    content: Content,
}

impl Fetched {
    pub fn new(tip: Option<Sha>, content: Content) -> Self {
        Self { tip, content }
    }

    pub fn tip(&self) -> Option<&Sha> {
        self.tip.as_ref()
    }

    pub fn content(&self) -> &Content {
        &self.content
    }
}
