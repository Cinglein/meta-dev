use std::fmt;

use syn::Path;

use super::workspace_types::WorkspaceTypes;
use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Callee {
    Method(Name),
    Associated { owner: Name, function: Name },
}

impl Callee {
    const SELF_TYPE: &str = "Self";

    pub fn associated(path: &Path, self_type: Option<&Name>, types: &WorkspaceTypes) -> Option<Callee> {
        let names: Vec<Name> = path.segments.iter().map(|segment| Name::from(&segment.ident)).collect();
        match names.as_slice() {
            [.., owner, function] if function.as_str().starts_with(char::is_lowercase) => {
                Self::resolved(owner, self_type)
                    .filter(|owner| types.declares(owner))
                    .map(|owner| Callee::Associated { owner, function: function.clone() })
            }
            _ => None,
        }
    }

    fn resolved(owner: &Name, self_type: Option<&Name>) -> Option<Name> {
        if owner == Self::SELF_TYPE { self_type.cloned() } else { Some(owner.clone()) }
    }
}

impl fmt::Display for Callee {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Callee::Method(method) => write!(formatter, "{method}"),
            Callee::Associated { owner, function } => write!(formatter, "{owner}::{function}"),
        }
    }
}
