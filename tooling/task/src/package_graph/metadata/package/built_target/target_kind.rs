use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetKind {
    Example,
    #[serde(other)]
    Other,
}
