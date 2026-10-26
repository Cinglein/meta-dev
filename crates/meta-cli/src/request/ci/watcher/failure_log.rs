use super::gh::Gh;
use crate::request::ci::{CiStatus, SessionDir, SessionId};
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::output::Output;
use crate::request::project::Project;

#[derive(Debug)]
pub struct FailureLog;

impl FailureLog {
    pub fn excerpt(project: &Project, session: &SessionId) -> Result<Output, CliFailure> {
        let dir =
            SessionDir::for_session(project, session).ok_or_else(|| CliFailure::raised::<Setup>("no git directory"))?;
        let pushed = dir.pushed().map_err(CliFailure::raised::<Setup>)?.commits();
        let failed = pushed.iter().find(|commit| dir.status_of(commit) == Some(CiStatus::Failed));
        let commit =
            failed.or(pushed.first()).ok_or_else(|| CliFailure::raised::<Setup>("no CI tracked for this session"))?;
        let settings = project.config().ci_settings();
        let deadline = std::time::Instant::now() + settings.remaining(std::time::Duration::ZERO).unwrap_or_default();
        let raw = Gh::new(settings.program().clone(), project.root().to_path_buf(), deadline).failed_logs(commit);
        Ok(Output::Listed(raw.excerpt(settings.log_window())))
    }
}
