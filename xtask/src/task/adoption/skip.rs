use std::fmt;

use crate::task::adoption::scalar::Scalar;
use crate::task::adoption::trigger::Trigger;
use crate::task::adoption::yaml_node::YamlNode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Skip {
    Untriggered(Trigger),
    JobCondition(Scalar),
    StepCondition(Scalar),
    Tolerated,
    Fallback,
    Unpiped,
}

impl Skip {
    const TOLERATED: &str = "continue-on-error";
    const NOT_TOLERATED: &str = "false";

    pub fn tolerated(node: &YamlNode) -> Option<Skip> {
        node.child(Self::TOLERATED)
            .filter(|tolerated| &**tolerated.scalar() != Self::NOT_TOLERATED)
            .map(|_| Skip::Tolerated)
    }
}

impl fmt::Display for Skip {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Skip::Untriggered(trigger) => write!(formatter, "its workflow does not run on {trigger}"),
            Skip::JobCondition(condition) => write!(formatter, "job `if: {condition}`"),
            Skip::StepCondition(condition) => write!(formatter, "step `if: {condition}`"),
            Skip::Tolerated => formatter.write_str("`continue-on-error`"),
            Skip::Fallback => formatter.write_str("a `||` fallback"),
            Skip::Unpiped => formatter.write_str("a pipe without `shell: bash`, so no pipefail"),
        }
    }
}
