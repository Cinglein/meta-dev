use std::time::Instant;

use meta_log::LogStore as _;
use meta_schema::Event;
use meta_state::State;

use super::hook_input::HookInput;
use crate::request::Request;
use crate::request::live_github::LiveGithub;
use crate::request::output::Listed;
use crate::request::project::Project;
use crate::request::writing::Writing;

#[derive(Debug)]
pub struct Linking;

impl Linking {
    pub fn after(input: &HookInput, known: Option<&Project>, deadline: Instant) -> Option<Listed> {
        let found = Project::try_from(input.publishing()?.as_path()).ok()?;
        let project = known.filter(|known| known.root() == found.root()).unwrap_or(&found);
        let branch = project.unprotected_branch(deadline)?;
        let task = branch.task()?;
        let pulls = LiveGithub::for_project(project).pulls_from(&branch);
        pulls.first()?;
        let log = project.log(deadline).ok()?;
        let links = State::fold(log.read().ok()?.records()).unlinked(task, &pulls);
        let actor = Request::actor().ok()?;
        let written = links.into_iter().map(|link| {
            let writing = Writing::new(&log, actor.clone(), Event::TaskLinked(link)).entered();
            writing.map_or_else(Listed::new, Listed::new)
        });
        written.reduce(Listed::then)
    }
}
