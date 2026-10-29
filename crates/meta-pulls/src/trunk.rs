use meta_schema::PullRequest;

use crate::live_pull::BaseBranch;
use crate::stop::Stop;
use crate::strictness::Strictness;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trunk {
    default: BaseBranch,
    strictness: Strictness,
}

impl Trunk {
    #[must_use]
    pub fn new(default: BaseBranch, strictness: Strictness) -> Self {
        Self { default, strictness }
    }

    pub(crate) fn barred(&self, pull_request: PullRequest, base: &BaseBranch) -> Option<Stop> {
        let off = (*base != self.default).then(|| Stop::OffTrunk { pull_request, base: base.clone() });
        off.or_else(|| self.strictness.loose().map(|()| Stop::Unguarded(self.default.clone())))
    }
}
