use crate::request::ci::Commit;
use crate::request::project::BranchName;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Checkout {
    head: Option<Commit>,
    branch: Option<BranchName>,
}

impl Checkout {
    const HEADS: &'static str = "refs/heads/";
    const DETACHED: &'static str = "HEAD";
    const ATTACHED: &'static str = "HEAD -> ";
    const DECORATIONS: &'static str = ", ";

    pub fn decorated(head: &Commit, decorations: &str) -> Option<Self> {
        let named = decorations.split(Self::DECORATIONS).find_map(|decoration| {
            (decoration == Self::DETACHED).then_some(decoration).or_else(|| decoration.strip_prefix(Self::ATTACHED))
        })?;
        let branch = named.strip_prefix(Self::HEADS).and_then(|name| name.parse().ok());
        Some(Self { head: Some(head.clone()), branch })
    }

    pub fn head(&self) -> Option<&Commit> {
        self.head.as_ref()
    }

    pub fn branch(&self) -> Option<&BranchName> {
        self.branch.as_ref()
    }
}
