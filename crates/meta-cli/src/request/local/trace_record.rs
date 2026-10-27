use std::path::PathBuf;

use meta_trace::{CommandWord, Retention, Trace};

use super::LocalClone;
use super::local_settings::LocalSettings;
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::output::{Listed, Output};
use crate::request::project::Project;

#[derive(Debug)]
pub struct TraceRecord {
    dir: PathBuf,
    kept: Retention,
}

impl TraceRecord {
    const DIR: &'static str = "meta-local/traces";
    const NONE_YET: &'static str = "trace: none yet; run a meta command with META_TRACE=on";

    pub fn in_project(project: &Project) -> Result<Self, CliFailure> {
        let clone = LocalClone::try_from(project)?;
        let kept = LocalSettings::read(clone.checkout())?.trace_kept();
        Ok(Self { dir: clone.git_dir().as_ref().join(Self::DIR), kept })
    }

    pub fn kept_here(trace: &Trace, word: &CommandWord) -> Result<(), CliFailure> {
        let here = std::env::current_dir().map_err(CliFailure::raised::<Setup>)?;
        let record = Self::in_project(&Project::try_from(here.as_path())?)?;
        trace.kept(&record.dir, word, record.kept).map(drop).map_err(CliFailure::raised::<Setup>)
    }

    pub fn last(&self) -> Result<Output, CliFailure> {
        let latest = Trace::latest(&self.dir).map_err(CliFailure::raised::<Setup>)?;
        let trace = latest.ok_or_else(|| CliFailure::raised::<Setup>(Self::NONE_YET))?;
        Ok(Output::Listed(Listed::new(trace)))
    }
}
