use std::fmt;

use syn::Ident;

use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VagueName(Ident);

impl VagueName {
    const MIN_LENGTH: usize = 3;
    const VAGUE: [&str; 24] = [
        "with", "from", "into", "for", "get", "set", "make", "check", "build", "create", "init", "process", "handle",
        "helper", "util", "data", "value", "item", "thing", "execute", "perform", "update", "compute", "manage",
    ];

    pub fn name(&self) -> &Ident {
        &self.0
    }
}

impl TryFrom<&Ident> for VagueName {
    type Error = Name;

    fn try_from(name: &Ident) -> Result<VagueName, Name> {
        let text = name.to_string();
        (text.len() < Self::MIN_LENGTH || Self::VAGUE.contains(&text.as_str()))
            .then(|| VagueName(name.clone()))
            .ok_or_else(|| Name::from(name))
    }
}

impl fmt::Display for VagueName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {} names nothing; say what it does", self.0)
    }
}
