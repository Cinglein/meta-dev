use meta_schema::Actor;
use serde::Deserialize;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchPolicy {
    #[serde(default)]
    issuers: Vec<Actor>,
}

impl DispatchPolicy {
    pub fn issuer(&self, actor: Actor) -> Option<Actor> {
        Some(actor).filter(|actor| self.issuers.contains(actor))
    }
}
