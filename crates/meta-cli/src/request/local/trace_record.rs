use std::io::Write as _;
use std::path::Path;

use meta_trace::{Recording, TraceDir, Verb};

use super::LocalClone;
use super::local_settings::LocalSettings;
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::environment::Environment;
use crate::request::git_link::GitLink;
use crate::request::output::Output;
use crate::request::project::Project;
use crate::request::trace_view::TraceView;

#[derive(Debug)]
pub struct TraceRecord(Option<Recording>);

impl TraceRecord {
    const DIR: &'static str = "meta-local/traces";
    const SWITCH: &'static str = "META_TRACE";
    const OFF: &'static str = "off";
    const NONE_YET: &'static str = "trace: none yet";

    #[must_use]
    pub fn started(args: &[String], environment: &Environment) -> Self {
        let verb = args.get(1).and_then(|word| word.parse().ok()).unwrap_or(Verb::Other);
        let switched = environment.variable(Self::SWITCH).filter(|switch| *switch == Self::OFF);
        Self(
            switched
                .map_or_else(|| Some(verb).filter(|verb| *verb != Verb::Trace).and_then(Recording::started), |_| None),
        )
    }

    pub fn kept(self, environment: &Environment) {
        drop(std::io::stdout().flush());
        let git_dir = Project::located(environment.directory()).and_then(|root| GitLink::common_dir(&root));
        let kept = self.0.zip(git_dir).map(|(recording, git_dir)| {
            let retention = LocalSettings::read(git_dir.parent().unwrap_or(Path::new("")))?.trace();
            recording.kept(&Self::dir(&git_dir), retention).map_err(CliFailure::raised::<Setup>)
        });
        kept.and_then(Result::err).into_iter().for_each(|failure| eprintln!("meta: trace not kept: {failure}"));
    }

    pub fn shown(clone: &LocalClone, view: TraceView) -> Result<Output, CliFailure> {
        let trace = Self::dir(clone.git_dir().as_ref()).latest(view.verb()).map_err(CliFailure::raised::<Setup>)?;
        let trace = trace.ok_or_else(|| CliFailure::raised::<Setup>(Self::NONE_YET))?;
        Ok(Output::Listed(view.shown(&trace)?))
    }

    fn dir(git_dir: &Path) -> TraceDir {
        TraceDir::new(git_dir.join(Self::DIR))
    }
}
