use std::collections::{BTreeMap, BTreeSet};

use crate::field_name::FieldName;
use crate::method_key::MethodKey;
use crate::method_name::MethodName;
use crate::method_use::MethodUse;
use crate::usage_node::UsageNode;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Resolution {
    fields: BTreeSet<FieldName>,
    accessors: BTreeMap<MethodName, BTreeSet<FieldName>>,
    methods: BTreeMap<MethodName, BTreeSet<MethodKey>>,
}

impl Resolution {
    #[must_use]
    pub fn new(fields: BTreeSet<FieldName>, methods: &[&MethodUse]) -> Resolution {
        let accessors = methods
            .iter()
            .filter_map(|method| method.accessed().map(|field| (method.key().name().clone(), field.clone())))
            .fold(BTreeMap::new(), |mut accessors: BTreeMap<MethodName, BTreeSet<FieldName>>, (name, field)| {
                accessors.entry(name).or_default().insert(field);
                accessors
            });
        let methods = methods.iter().filter(|method| method.accessed().is_none()).map(|method| method.key()).fold(
            BTreeMap::new(),
            |mut methods: BTreeMap<MethodName, BTreeSet<MethodKey>>, key| {
                methods.entry(key.name().clone()).or_default().insert(key.clone());
                methods
            },
        );
        Resolution { fields, accessors, methods }
    }

    #[must_use]
    pub fn fields(&self) -> Vec<UsageNode> {
        self.fields.iter().cloned().map(UsageNode::Field).collect()
    }

    #[must_use]
    pub fn called(&self, name: &MethodName) -> Vec<UsageNode> {
        let accessed = self.accessors.get(name).into_iter().flatten().cloned().map(UsageNode::Field);
        let owned = self.methods.get(name).into_iter().flatten().cloned().map(UsageNode::Own);
        accessed.chain(owned).collect()
    }
}
