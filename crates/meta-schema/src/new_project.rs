use std::fmt;

use serde::{Deserialize, Serialize};

use crate::division::Division;
use crate::entity::Project;
use crate::id::Id;
use crate::text::Text;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewProject {
    project: Id<Project>,
    text: Text,
    division: Division,
}

impl fmt::Display for NewProject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} @{}", self.project, self.text, self.division)
    }
}
