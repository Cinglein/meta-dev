mod open_repository;

use std::collections::HashMap;

use meta_pulls::{BaseBranch, LivePull};
use meta_schema::PullRequest;
use serde::Deserialize;

use crate::live_data::LiveData;
use crate::pull_alias::PullAlias;
use crate::rulebook::Rulebook;
use crate::trunk::Trunk;
use crate::trunk_run::TrunkRun;
use crate::trunk_verdict::TrunkVerdict;
use crate::workflow_name::WorkflowName;
use open_repository::OpenRepository;

#[derive(Clone, Debug, Deserialize)]
pub struct Pulls {
    open: OpenRepository,
    #[serde(default)]
    linked: HashMap<PullAlias, Option<LivePull>>,
    #[serde(default)]
    trunk: Option<Trunk>,
}

impl Pulls {
    #[must_use]
    pub fn bases(&self) -> Vec<BaseBranch> {
        let mut bases: Vec<BaseBranch> = self
            .linked
            .values()
            .flatten()
            .chain(self.open.pull_requests().nodes())
            .map(|pull| pull.base().clone())
            .collect();
        bases.sort_by_key(ToString::to_string);
        bases.dedup();
        bases
    }

    #[must_use]
    pub fn ruled(self, rules: Rulebook) -> Option<LiveData> {
        self.bases().iter().all(|base| rules.required(base).is_some()).then(|| LiveData::new(self, rules))
    }

    pub fn pull(&self, pull_request: PullRequest) -> Option<&LivePull> {
        let linked = self.linked.get(&PullAlias::new(pull_request)).and_then(Option::as_ref);
        linked.or_else(|| self.open.pull_requests().nodes().find(|pull| pull.number() == pull_request))
    }

    #[must_use]
    pub fn trunk_run(&self, workflow: WorkflowName) -> TrunkRun {
        let verdict = self.trunk.as_ref().map_or(TrunkVerdict::Unknown, |trunk| trunk.verdict(&workflow));
        TrunkRun::new(workflow, verdict)
    }

    #[must_use]
    pub fn open(&self) -> &OpenRepository {
        &self.open
    }
}
