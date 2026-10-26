use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use meta_pull_query::{LiveQuery, LiveReply};
use meta_schema::PullRequest;
use meta_status::{Github, GithubFailure};

use crate::request::project::{BranchName, ProgramName, Project};
use crate::request::timed_command::TimedCommand;

#[derive(Clone, Debug)]
pub struct LiveGithub {
    program: ProgramName,
    root: PathBuf,
    deadline: Instant,
}

impl LiveGithub {
    const GRAPHQL: [&'static str; 6] = ["api", "graphql", "-F", "owner={owner}", "-F", "name={repo}"];
    const QUERY: &'static str = "-f";
    const HEADED: [&'static str; 3] = ["pr", "list", "--head"];
    const NUMBERS: [&'static str; 4] = ["--json", "number", "--jq", ".[].number"];

    pub fn new(program: ProgramName, root: PathBuf, deadline: Instant) -> Self {
        Self { program, root, deadline }
    }

    pub fn for_project(project: &Project) -> Self {
        let settings = project.config().ci_settings();
        Self::new(settings.program().clone(), project.root().to_path_buf(), settings.remote_deadline())
    }

    pub fn read(&self, query: &LiveQuery) -> Github {
        let mut command = Command::new(&self.program);
        command.args(Self::GRAPHQL).arg(Self::QUERY).arg(query.field()).current_dir(&self.root);
        TimedCommand::new(command, self.deadline).finished().map_or_else(
            |failure| Github::Unreached(GithubFailure::new(failure)),
            |finished| {
                let reply = serde_json::from_str::<LiveReply>(finished.output()).ok().and_then(LiveReply::answered);
                reply.map_or_else(|| Github::Unreached(GithubFailure::new(finished.complaint())), |_| {
                    Github::Unreached(GithubFailure::new(finished.complaint()))
                })
            },
        )
    }

    pub fn pulls_from(&self, branch: &BranchName) -> Vec<PullRequest> {
        let mut command = Command::new(&self.program);
        command.args(Self::HEADED).arg(branch).args(Self::NUMBERS).current_dir(&self.root);
        let listed = TimedCommand::new(command, self.deadline).stdout().unwrap_or_default();
        listed.lines().filter_map(|line| line.parse().ok()).collect()
    }
}

#[cfg(test)]
mod tests;
