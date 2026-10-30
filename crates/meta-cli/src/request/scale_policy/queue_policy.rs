use meta_schema::Actor;
use serde::Deserialize;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueuePolicy {
    #[serde(default)]
    landers: Vec<Actor>,
}

impl QueuePolicy {
    pub fn lander(&self, announced: Option<Actor>) -> Option<Actor> {
        announced.filter(|actor| self.landers.contains(actor))
    }
}

#[cfg(test)]
mod tests {
    use super::QueuePolicy;

    const POLICY: &str = "landers = [\"owner\", \"claude-lead\"]";
    const ACTORS: [&str; 2] = ["claude-lead", "claude-agent"];

    #[test]
    fn only_an_announced_actor_on_the_landers_list_may_land() {
        let policy: QueuePolicy = toml::from_str(POLICY).unwrap();
        let [lead, agent] = ACTORS.map(|name| policy.lander(name.parse().ok()));
        assert!(lead.is_some() && agent.is_none() && policy.lander(None).is_none());
    }
}
