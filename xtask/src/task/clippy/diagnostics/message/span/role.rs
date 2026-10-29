use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "bool")]
pub(super) enum Role {
    Primary,
    Secondary,
}

impl From<bool> for Role {
    fn from(primary: bool) -> Role {
        if primary { Role::Primary } else { Role::Secondary }
    }
}
