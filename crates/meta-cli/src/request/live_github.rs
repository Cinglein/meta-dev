use std::ffi::OsStr;
use std::num::NonZeroUsize;
use std::panic::resume_unwind;
use std::path::PathBuf;
use std::time::Instant;

use meta_pull_query::{LiveData, LiveQuery, Rulebook, WorkflowName};
use meta_pulls::{BaseBranch, RequiredChecks, RuleSource};
use meta_run::{EnvPolicy, Run};
use meta_schema::PullRequest;
use meta_status::{Github, GithubFailure};

use crate::request::project::{BranchName, ProgramName, Project};
use crate::request::repository::Repository;
use open_pages::OpenPages;
use unanswered::Unanswered;

mod open_pages;

#[derive(Clone, Debug)]
pub struct LiveGithub {
    program: ProgramName,
    root: PathBuf,
    deadline: Instant,
    workflows: Vec<WorkflowName>,
    repository: Option<Repository>,
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
    const REPOSITORY: &'static str = "GH_REPO";

    pub fn new(program: ProgramName, root: PathBuf, deadline: Instant, workflows: Vec<WorkflowName>) -> Self {
        Self { program, root, deadline, workflows, repository: None }
    }

    pub fn for_project(project: &Project) -> Self {
        let settings = project.ci_settings();
        let workflows = project.config().required_workflows().to_vec();
        let github =
            Self::new(settings.program().clone(), project.root().to_path_buf(), settings.remote_deadline(), workflows);
        github.naming(project.repository().ok())
    }

    #[must_use]
    pub fn naming(self, repository: Option<Repository>) -> Self {
        Self { repository, ..self }
    }

    #[must_use]
    pub fn within(self, overall: Instant) -> Self {
        Self { deadline: self.deadline.min(overall), ..self }
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
        let asked: Vec<(BaseBranch, RuleSource)> =
            pulls.bases().into_iter().flat_map(|base| RuleSource::BOTH.map(|source| (base.clone(), source))).collect();
        let width = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
        let rules: Rulebook = asked.chunks(width).flat_map(|chunk| self.together(chunk)).collect::<Result<_, _>>()?;
        let data = pulls.ruled(rules).ok_or_else(|| GithubFailure::new(Self::UNRULED))?;
        Ok(data.requiring(self.workflows.clone()))
    }

    fn together(&self, chunk: &[(BaseBranch, RuleSource)]) -> Vec<Result<(BaseBranch, RequiredChecks), GithubFailure>> {
        std::thread::scope(|scope| {
            let read =
                |(base, source): &(BaseBranch, RuleSource)| self.required(*source, base).map(|got| (base.clone(), got));
            let running: Vec<_> = chunk.iter().map(|asked| scope.spawn(move || read(asked))).collect();
            running.into_iter().map(|run| run.join().unwrap_or_else(|panic| resume_unwind(panic))).collect()
        })
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
        let run = Run::new(&self.program, EnvPolicy::Inherit).args(words).dir(&self.root);
        self.repository.iter().fold(run, |run, repository| run.var(Self::REPOSITORY, repository.to_string()))
    }
}

#[cfg(test)]
mod tests;
mod unanswered;
