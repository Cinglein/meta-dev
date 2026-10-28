use std::io;
use std::path::{Path, PathBuf};

use crate::retention::Retention;
use crate::trace::Trace;
use crate::trace_id::TraceId;
use crate::verb::Verb;

#[derive(Debug)]
pub struct TraceDir(PathBuf);

impl TraceDir {
    const EXTENSION: &'static str = "mtr";

    #[must_use]
    pub fn new(dir: PathBuf) -> Self {
        Self(dir)
    }

    pub fn kept(&self, trace: TraceId, spans: &Trace, retention: Retention) -> io::Result<()> {
        std::fs::create_dir_all(&self.0)?;
        std::fs::write(trace.file_in(&self.0, Self::EXTENSION), Vec::from(spans))?;
        retention.pruned(&self.listed()?)
    }

    pub fn latest(&self, verb: Option<Verb>) -> io::Result<Option<Trace>> {
        let listed = self.listed()?;
        let traces = listed.chunk_by(|newer, older| Self::trace_of(newer) == Self::trace_of(older));
        let read = traces.map(|files| files.iter().filter_map(|file| std::fs::read(file).ok()));
        Ok(read
            .map(|files| files.map(|bytes| Trace::from(bytes.as_slice())).collect::<Trace>())
            .find(|trace| trace.started_by(verb)))
    }

    fn listed(&self) -> io::Result<Vec<PathBuf>> {
        let absent = |failure: io::Error| (failure.kind() == io::ErrorKind::NotFound).then_some(None).ok_or(failure);
        let entries = std::fs::read_dir(&self.0).map(Some).or_else(absent)?.into_iter().flatten().flatten();
        let mut listed: Vec<PathBuf> = entries
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == Self::EXTENSION))
            .collect();
        listed.sort_by(|newer, older| older.cmp(newer));
        Ok(listed)
    }

    fn trace_of(file: &Path) -> Option<&str> {
        file.file_name()?.to_str()?.get(..TraceId::SHOWN_LEN)
    }
}
