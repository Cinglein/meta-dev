use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Section {
    Whole,
    Frontend,
    Codegen,
    #[serde(other)]
    Other,
}
