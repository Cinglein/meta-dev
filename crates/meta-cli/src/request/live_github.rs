use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Instant;

use meta_pull_query::{LiveData, LiveQuery, Rulebook, WorkflowName};
use meta_pulls::{BaseBranch, RequiredChecks, RuleSource};
use meta_run::{EnvPolicy, Run};
use meta_schema::PullRequest;
use meta_status::{Github, GithubFailure};

use crate::request::project::{BranchName, ProgramName, Project};
use open_pages::OpenPages;
use unanswered::Unanswered;

mod open_pages;

#[derive(Clone, Debug)]
pub struct LiveGithub {
    program: ProgramName,
    root: PathBuf,
    deadline: Instant,
    workflows: Vec<WorkflowName>,
}

impl LiveGithub {
    const GRAPHQL: [&'static str; 6] = ["api", "graphql", "-F", "owner={owner}", "-F", "name={repo}"];
    const QUERY: &'static str = "-f";
    const API: &'static str = "api";
    const FILTER: &'static str = "--jq";
    const HEADED: [&'static str; 3] = ["pr", "list", "--head"];
    const SAME_REPOSITORY: [&'static str; 4] =
        ["--json", "number,isCrossRepository", "--jq", ".[] | select(.isCrossRepository | not) | .number"];
    const UNRULED: &'static str = "a base branch's rules were not read";

    pub fn new(program: ProgramName, root: PathBuf, deadline: Instant, workflows: Vec<WorkflowName>) -> Self {
        Self { program, root, deadline, workflows }
    }

    pub fn for_project(project: &Project) -> Self {
        let settings = project.ci_settings();
        let workflows = project.config().required_workflows().to_vec();
        Self::new(settings.program().clone(), project.root().to_path_buf(), settings.remote_deadline(), workflows)
    }

    #[must_use]
    pub fn within(self, overall: Instant) -> Self {
        Self { program: self.program, root: self.root, deadline: self.deadline.min(overall), workflows: self.workflows }
    }

    pub fn read(&self, query: &LiveQuery) -> Github {
        self.live(query).map_or_else(Github::Unreached, Github::Reached)
    }

    pub fn pulls_from(&self, branch: &BranchName) -> Vec<PullRequest> {
        let words = Self::HEADED.map(OsStr::new).into_iter().chain([branch.as_ref()]);
        let listed = self.run(words.chain(Self::SAME_REPOSITORY.map(OsStr::new))).stdout(self.deadline);
        listed.unwrap_or_default().lines().filter_map(|line| line.parse().ok()).collect()
    }

    fn live(&self, query: &LiveQuery) -> Result<LiveData, GithubFailure> {
        let pulls = OpenPages::new(self, query).read()?;
        let rules: Rulebook = pulls
            .bases()
            .into_iter()
            .map(|base| self.rules(&base, pulls.protection(&base)).map(|required| (base, required)))
            .collect::<Result<_, _>>()?;
        let data = pulls.ruled(rules).ok_or_else(|| GithubFailure::new(Self::UNRULED))?;
        Ok(data.requiring(self.workflows.clone()))
    }

    fn rules(&self, base: &BaseBranch, protection: Option<RuleSource>) -> Result<RequiredChecks, GithubFailure> {
        std::iter::once(RuleSource::Rulesets)
            .chain(protection)
            .try_fold(RequiredChecks::default(), |all, source| Ok(all.united(self.required(source, base)?)))
    }

    fn required(&self, source: RuleSource, base: &BaseBranch) -> Result<RequiredChecks, GithubFailure> {
        let endpoint = base.endpoint(source);
        let words = [Self::API, endpoint.as_str(), Self::FILTER, source.filter()].map(OsStr::new);
        let ran = self.run(words.into_iter()).ran(self.deadline).map_err(GithubFailure::new)?;
        match (source, Unanswered::new(&ran).not_found()) {
            (RuleSource::Protection, Some(())) => Ok(RequiredChecks::default()),
            (RuleSource::Rulesets | RuleSource::Protection, _) => {
                let printed = ran.succeeded().map_err(GithubFailure::new)?;
                serde_json::from_str(&printed).map_err(GithubFailure::new)
            }
        }
    }

    fn run<'words>(&self, words: impl Iterator<Item = &'words OsStr>) -> Run {
        Run::new(&self.program, EnvPolicy::Inherit).args(words).dir(&self.root)
    }
}

#[cfg(test)]
mod tests;
mod unanswered;
