use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use meta_pull_query::{LiveData, LiveQuery, LiveReply, Pulls, Rulebook};
use meta_pulls::{BaseBranch, RequiredChecks};
use meta_schema::PullRequest;
use meta_status::{Github, GithubFailure};

use crate::request::project::{BranchName, ProgramName, Project};
use crate::request::timed_command::{Finished, TimedCommand};

#[derive(Clone, Debug)]
pub struct LiveGithub {
    program: ProgramName,
    root: PathBuf,
    deadline: Instant,
}

impl LiveGithub {
    const GRAPHQL: [&'static str; 6] = ["api", "graphql", "-F", "owner={owner}", "-F", "name={repo}"];
    const QUERY: &'static str = "-f";
    const API: &'static str = "api";
    const REQUIRED: [&'static str; 2] =
        ["--jq", "[.[] | select(.type == \"required_status_checks\") | .parameters.required_status_checks[].context]"];
    const HEADED: [&'static str; 3] = ["pr", "list", "--head"];
    const SAME_REPOSITORY: [&'static str; 4] =
        ["--json", "number,isCrossRepository", "--jq", ".[] | .number"];
    const UNRULED: &'static str = "a base branch's rules were not read";

    pub fn new(program: ProgramName, root: PathBuf, deadline: Instant) -> Self {
        Self { program, root, deadline }
    }

    pub fn for_project(project: &Project) -> Self {
        let settings = project.config().ci_settings();
        Self::new(settings.program().clone(), project.root().to_path_buf(), settings.remote_deadline())
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
        let pulls = self.pulls(query)?;
        let rules: Rulebook = pulls
            .bases()
            .into_iter()
            .map(|base| self.rules(&base).map(|required| (base, required)))
            .collect::<Result<_, _>>()?;
        pulls.ruled(rules).ok_or_else(|| GithubFailure::new(Self::UNRULED))
    }

    fn pulls(&self, query: &LiveQuery) -> Result<Pulls, GithubFailure> {
        let field = query.field();
        let words = Self::GRAPHQL.map(OsStr::new).into_iter().chain([OsStr::new(Self::QUERY), OsStr::new(&field)]);
        let finished = self.ran(words).map_err(GithubFailure::new)?;
        let reply = serde_json::from_str::<LiveReply>(finished.output()).ok().and_then(LiveReply::answered);
        reply.ok_or_else(|| GithubFailure::new(finished))
    }

    fn rules(&self, base: &BaseBranch) -> Result<RequiredChecks, GithubFailure> {
        let endpoint = base.rules_endpoint();
        let words = [OsStr::new(Self::API), OsStr::new(&endpoint)].into_iter().chain(Self::REQUIRED.map(OsStr::new));
        let printed = self.ran(words).and_then(Finished::succeeded).map_err(GithubFailure::new)?;
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
