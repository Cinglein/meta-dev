use std::ffi::OsStr;

use meta_pull_query::{LiveQuery, LiveReply, Pulls};
use meta_status::GithubFailure;

use super::LiveGithub;
use super::unanswered::Unanswered;

#[derive(Clone, Copy, Debug)]
pub struct OpenPages<'github> {
    github: &'github LiveGithub,
    query: &'github LiveQuery,
}

impl<'github> OpenPages<'github> {
    pub fn new(github: &'github LiveGithub, query: &'github LiveQuery) -> Self {
        Self { github, query }
    }

    pub fn read(&self) -> Result<Pulls, GithubFailure> {
        self.page(self.query).and_then(|first| self.rest(first))
    }

    fn rest(&self, pulls: Pulls) -> Result<Pulls, GithubFailure> {
        match pulls.following() {
            Some(next) => self.page(&next).and_then(|page| self.rest(pulls.continued(page))),
            None => Ok(pulls),
        }
    }

    fn page(&self, query: &LiveQuery) -> Result<Pulls, GithubFailure> {
        let field = query.field();
        let cursor = query.cursor_field();
        let after = cursor.iter().flat_map(|variable| [OsStr::new(LiveGithub::QUERY), OsStr::new(variable)]);
        let first = [OsStr::new(LiveGithub::QUERY), OsStr::new(&field)];
        let words = LiveGithub::GRAPHQL.map(OsStr::new).into_iter().chain(first).chain(after);
        let ran = self.github.run(words).ran(self.github.deadline).map_err(GithubFailure::new)?;
        let reply = serde_json::from_str::<LiveReply>(&ran.stdout().text()).ok().and_then(LiveReply::answered);
        reply.ok_or_else(|| GithubFailure::new(Unanswered::new(&ran)))
    }
}
