use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

use crate::argv::Argv;
use crate::golden::Golden;
use crate::golden_failure::GoldenFailure;
use crate::recording::Recording;

#[derive(Debug)]
pub struct Replay {
    recordings: PathBuf,
    trail: PathBuf,
}

impl Replay {
    pub const RECORDINGS: &'static str = "META_REPLAY";
    pub const TRAIL: &'static str = "META_REPLAY_TRAIL";

    #[must_use]
    pub fn answered(asked: Vec<String>) -> ExitCode {
        let place = |name| std::env::var_os(name).map(PathBuf::from).unwrap_or_default();
        let replay = Self { recordings: place(Self::RECORDINGS), trail: place(Self::TRAIL) };
        replay.recording(&Argv::from(asked)).map_or_else(
            |failure| {
                eprintln!("meta-replay: {failure}");
                ExitCode::FAILURE
            },
            |recording| recording.answered(),
        )
    }

    fn recording(&self, asked: &Argv) -> Result<Recording, GoldenFailure> {
        let trail = Golden::new(&self.trail).text().unwrap_or_default();
        let earlier =
            trail.lines().filter_map(|line| line.parse::<Argv>().ok()).filter(|seen| seen.answers(asked)).count();
        let unwritable = |source| GoldenFailure::Unreadable { path: self.trail.clone(), source };
        let file = std::fs::OpenOptions::new().create(true).append(true).open(&self.trail).map_err(unwritable)?;
        (&file).write_fmt(format_args!("{asked}\n")).map_err(unwritable)?;
        let mut answers = Recording::all_in(&self.recordings)?;
        answers.retain(|recording| recording.argv().answers(asked));
        let chosen = answers.get(earlier).or(answers.last()).cloned();
        chosen.ok_or_else(|| GoldenFailure::Unanswered(asked.clone()))
    }
}
