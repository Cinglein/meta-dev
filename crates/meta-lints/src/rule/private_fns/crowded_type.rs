use std::fmt;

use crate::rule::name::Name;
use crate::rule::private_fns::private_fn_limit::PrivateFnLimit;
use crate::rule::private_fns::self_type::SelfType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrowdedType {
    self_type: SelfType,
    names: Vec<Name>,
    limit: PrivateFnLimit,
}

impl CrowdedType {
    pub fn new(self_type: SelfType, names: Vec<Name>, limit: PrivateFnLimit) -> CrowdedType {
        CrowdedType { self_type, names, limit }
    }
}

impl fmt::Display for CrowdedType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self.names.iter().map(Name::as_str).collect();
        write!(
            formatter,
            "{} has {} private fns, at most {} allowed: {}",
            self.self_type,
            self.names.len(),
            self.limit,
            names.join(", ")
        )
    }
}
