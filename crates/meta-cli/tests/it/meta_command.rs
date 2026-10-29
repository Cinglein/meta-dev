use std::fmt;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MetaCommand<'words> {
    words: &'words [&'words str],
    run_index: usize,
}

impl MetaCommand<'_> {
    const PARENT_SID: &str = "GIT_TRACE2_PARENT_SID";
    const UNNAMED: &str = "none";

    pub fn tagged(words: &[&str]) -> Command {
        static RUNS: AtomicUsize = AtomicUsize::new(0);
        let tag = MetaCommand { words, run_index: RUNS.fetch_add(1, Ordering::Relaxed) };
        let mut command = Command::new(env!("CARGO_BIN_EXE_meta"));
        command.args(words).env(Self::PARENT_SID, tag.to_string());
        command
    }
}

impl fmt::Display for MetaCommand<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let named = self.words.first().copied().unwrap_or(Self::UNNAMED);
        write!(formatter, "meta-{named}.{}.{}", std::process::id(), self.run_index)
    }
}
