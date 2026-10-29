use std::cmp::Reverse;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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
        let file = trace.file_in(&self.0, Self::EXTENSION);
        let written = std::fs::create_dir_all(&self.0).and_then(|()| std::fs::write(&file, Vec::from(spans)));
        written.as_ref().err().into_iter().for_each(|_| drop(std::fs::remove_file(&file)));
        let pruned = self.listed().and_then(|traces| retention.pruned(&traces));
        written.and(pruned)
    }

    pub fn latest(&self, verb: Option<Verb>) -> io::Result<Option<Trace>> {
        let read = |files: &Vec<PathBuf>| {
            let read = files.iter().filter_map(|file| std::fs::read(file).ok());
            read.map(|bytes| Trace::from(bytes.as_slice())).collect::<Trace>()
        };
        Ok(self.listed()?.iter().map(read).find(|trace| trace.started_by(verb)))
    }

    fn listed(&self) -> io::Result<Vec<Vec<PathBuf>>> {
        let absent = |failure: io::Error| (failure.kind() == io::ErrorKind::NotFound).then_some(None).ok_or(failure);
        let entries = std::fs::read_dir(&self.0).map(Some).or_else(absent)?.into_iter().flatten().flatten();
        let mut files: Vec<PathBuf> = entries
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == Self::EXTENSION))
            .collect();
        files.sort();
        let mut traces: Vec<Vec<PathBuf>> = files
            .chunk_by(|one, other| Self::trace_of(one) == Self::trace_of(other))
            .map(<[PathBuf]>::to_vec)
            .collect();
        traces.sort_by_cached_key(|files| Reverse((Self::written(files), files.first().cloned())));
        Ok(traces)
    }

    fn written(files: &[PathBuf]) -> Option<SystemTime> {
        files.iter().filter_map(|file| std::fs::metadata(file).and_then(|meta| meta.modified()).ok()).max()
    }

    fn trace_of(file: &Path) -> Option<&str> {
        file.file_name()?.to_str()?.get(..TraceId::SHOWN_LEN)
    }
}
