use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(from = "bool")]
pub enum Continuation {
    More,
    #[default]
    Complete,
}

impl From<bool> for Continuation {
    fn from(more: bool) -> Self {
        if more { Self::More } else { Self::Complete }
    }
}
