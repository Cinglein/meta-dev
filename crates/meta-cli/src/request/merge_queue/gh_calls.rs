use std::ffi::OsString;
use std::time::Instant;

use meta_pulls::{BaseBranch, GhRefusal, RuleSource, Stop, Strictness, Trunk};
use meta_run::{Port, PullArgv};

use crate::request::project::Project;

#[derive(Clone, Copy, Debug)]
pub struct GhCalls<'project> {
    project: &'project Project,
}

impl<'project> GhCalls<'project> {
    pub fn new(project: &'project Project) -> Self {
        Self { project }
    }

    pub fn called(self, call: PullArgv, operands: Vec<OsString>) -> Result<String, GhRefusal> {
        let settings = self.project.ci_settings();
        let run = Port::<PullArgv>::named(settings.program(), self.project.root()).run(call, operands);
        let run = self.project.repository().ok().iter().fold(run, |run, repository| repository.named(run));
        let printed = run.reported(Instant::now() + settings.gh_limit()).map_err(GhRefusal::new)?;
        printed.into_text().map_err(GhRefusal::new)
    }

    pub fn trunk(self) -> Result<Trunk, Stop> {
        let printed = self.called(PullArgv::DefaultBranch, Vec::new()).map_err(Stop::TrunkUnread)?;
        let Ok(default) = printed.parse::<BaseBranch>();
        let endpoint = vec![OsString::from(default.endpoint(RuleSource::Rulesets))];
        let strict = self.called(PullArgv::StrictRules, endpoint).map_err(Stop::TrunkUnread)?;
        let Ok(strictness) = strict.parse::<Strictness>();
        Ok(Trunk::new(default, strictness))
    }
}
