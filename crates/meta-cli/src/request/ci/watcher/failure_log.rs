use super::gh::Gh;
use crate::request::ci::{SessionDir, SessionId};
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::output::Output;
use crate::request::project::Project;

#[derive(Debug)]
pub struct FailureLog;

impl FailureLog {
    pub fn excerpt(project: &Project, session: &SessionId) -> Result<Output, CliFailure> {
        let dir =
            SessionDir::for_session(project, session).ok_or_else(|| CliFailure::raised::<Setup>("no git directory"))?;
        let commit = dir.tracked().ok_or_else(|| CliFailure::raised::<Setup>("no CI tracked for this session"))?;
        let settings = project.config().ci_settings();
        let raw = Gh::new(settings.program().clone(), project.root().to_path_buf()).failed_logs(&commit);
        Ok(Output::Listed(raw.excerpt(settings.log_window())))
    }
}
