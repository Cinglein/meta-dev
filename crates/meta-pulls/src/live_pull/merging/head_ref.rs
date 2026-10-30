mod head_target;

use serde::Deserialize;

use head_target::HeadTarget;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct HeadRef {
    target: HeadTarget,
}

impl HeadRef {
    pub fn target(&self) -> &HeadTarget {
        &self.target
    }
}
