use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use meta_pull_query::{LiveData, LiveQuery, Rulebook, WorkflowName};
use meta_pulls::{BaseBranch, RequiredChecks, RuleSource};
use meta_schema::PullRequest;
use meta_status::{Github, GithubFailure};

use crate::request::project::{BranchName, ProgramName, Project};
use crate::request::timed_command::{Finished, TimedCommand};
use open_pages::OpenPages;

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
        let config = project.config();
        let settings = config.ci_settings();
        let workflows = config.required_workflows().to_vec();
        Self::new(settings.program().clone(), project.root().to_path_buf(), settings.remote_deadline(), workflows)
    }

    pub fn read(&self, query: &LiveQuery) -> Github {
        self.live(query).map_or_else(Github::Unreached, Github::Reached)
    }

    pub fn pulls_from(&self, branch: &BranchName) -> Vec<PullRequest> {
        let words = Self::HEADED.map(OsStr::new).into_iter().chain([branch.as_ref()]);
        let listed = self.ran(words.chain(Self::SAME_REPOSITORY.map(OsStr::new))).and_then(Finished::succeeded);
        listed.unwrap_or_default().lines().filter_map(|line| line.parse().ok()).collect()
    }

    fn live(&self, query: &LiveQuery) -> Result<LiveData, GithubFailure> {
        let pulls = OpenPages::new(self, query).read()?;
        let rules: Rulebook = pulls
            .bases()
            .into_iter()
            .map(|base| self.rules(&base).map(|required| (base, required)))
            .collect::<Result<_, _>>()?;
        let data = pulls.ruled(rules).ok_or_else(|| GithubFailure::new(Self::UNRULED))?;
        Ok(data.requiring(self.workflows.clone()))
    }

    fn rules(&self, base: &BaseBranch) -> Result<RequiredChecks, GithubFailure> {
        RuleSource::ALL
            .into_iter()
            .try_fold(RequiredChecks::default(), |all, source| Ok(all.united(self.required(source, base)?)))
    }

    fn required(&self, source: RuleSource, base: &BaseBranch) -> Result<RequiredChecks, GithubFailure> {
        let endpoint = base.endpoint(source);
        let words = [Self::API, endpoint.as_str(), Self::FILTER, source.filter()].map(OsStr::new);
        let printed = self.ran(words.into_iter()).and_then(Finished::succeeded).map_err(GithubFailure::new)?;
        serde_json::from_str(&printed).map_err(GithubFailure::new)
    }

    fn ran<'words>(
        &self,
        words: impl Iterator<Item = &'words OsStr>,
    ) -> Result<Finished, crate::request::output::Listed> {
        let mut command = Command::new(&self.program);
        command.args(words).current_dir(&self.root);
        TimedCommand::new(command, self.deadline).finished()
    }
}

#[cfg(test)]
mod tests;
