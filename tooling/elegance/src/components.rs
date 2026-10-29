use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

use crate::listed::Listed;
use crate::method_name::MethodName;
use crate::tally::Tally;
use crate::usage_node::UsageNode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Components(Vec<Listed<MethodName>>);

impl Components {
    #[must_use]
    pub fn joined(neighbourhoods: impl IntoIterator<Item = BTreeSet<UsageNode>>) -> Components {
        let merged = neighbourhoods.into_iter().fold(Vec::<BTreeSet<UsageNode>>::new(), |components, next| {
            let (touching, apart): (Vec<BTreeSet<UsageNode>>, Vec<BTreeSet<UsageNode>>) =
                components.into_iter().partition(|component| !component.is_disjoint(&next));
            let joined = touching.into_iter().flatten().chain(next).collect();
            apart.into_iter().chain(std::iter::once(joined)).collect()
        });
        let mut methods: Vec<Listed<MethodName>> = merged
            .into_iter()
            .map(|component| Listed::new(component.into_iter().filter_map(UsageNode::method)))
            .collect();
        methods.sort();
        Components(methods)
    }

    #[must_use]
    pub fn is_split(&self) -> bool {
        Tally::counting(&self.0).reaches(Tally::PAIR)
    }

    #[must_use]
    pub fn most_first(&self, other: &Components) -> Ordering {
        other.0.len().cmp(&self.0.len())
    }
}

impl fmt::Display for Components {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.0.iter().map(|component| format!("[{component}]")).collect();
        write!(formatter, "{} components {}", self.0.len(), parts.join(" "))
    }
}
