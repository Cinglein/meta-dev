use super::gh::Gh;
use super::reach::Reach;
use super::session_commits::SessionCommits;
use crate::request::ci::SessionId;
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::output::Output;
use crate::request::project::Project;

#[derive(Debug)]
pub struct FailureLog;

impl FailureLog {
    pub fn excerpt(project: &Project, session: &SessionId) -> Result<Output, CliFailure> {
        let seen = SessionCommits::judged(project, session, Reach::Remote).map_err(CliFailure::raised::<Setup>)?;
        let pushed = seen.pushed();
        pushed.first().ok_or_else(|| CliFailure::raised::<Setup>("no CI tracked for this session"))?;
        let settings = project.config().ci_settings();
        let deadline = std::time::Instant::now() + settings.remaining(std::time::Duration::ZERO).unwrap_or_default();
        let raw = Gh::new(settings.program().clone(), project.root().to_path_buf(), deadline).failed_logs(pushed);
        Ok(Output::Listed(raw.excerpt(settings.log_window())))
    }
}
