use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GateStatus(String);

impl fmt::Display for GateStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::budget::Millis;
    use crate::policy::Policy;
    use settings::EmbeddedDefaults;

    const GATE: &str = "local-gate";

    #[test]
    fn the_checked_in_policy_names_the_gate_status() {
        let named = Policy::defaults().gate().map(ToString::to_string);
        assert_eq!(named.ok().as_deref(), Some(GATE));
        assert!(
            Policy::defaults()
                .gate_wait()
                .is_ok_and(|wait| wait > Millis::default())
        );
    }
}
