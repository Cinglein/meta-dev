mod claim;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::argv::Argv;
use crate::golden_failure::GoldenFailure;
use crate::places::Places;
use crate::recording::Recording;
use claim::Claim;

#[derive(Debug)]
pub struct Replay {
    recordings: PathBuf,
    trail: PathBuf,
}

impl Replay {
    pub const RECORDINGS: &'static str = "META_REPLAY";
    pub const TRAIL: &'static str = "META_REPLAY_TRAIL";
    const HERE: &'static str = ".";

    #[must_use]
    pub fn answered(asked: &[String]) -> ExitCode {
        let place = |name| std::env::var_os(name).map_or_else(|| PathBuf::from(Self::HERE), PathBuf::from);
        let replay = Self { recordings: place(Self::RECORDINGS), trail: place(Self::TRAIL) };
        let places = Places::new(&std::env::current_dir().unwrap_or_default(), Path::new(""));
        let asked = Argv::from(asked.iter().map(|word| places.hidden(word)).collect::<Vec<_>>());
        replay.recording(&asked).map_or_else(
            |failure| {
                eprintln!("meta-replay: {failure}");
                ExitCode::FAILURE
            },
            |recording| recording.answered(&places),
        )
    }

    fn recording(&self, asked: &Argv) -> Result<Recording, GoldenFailure> {
        let mut answers = Recording::all_in(&self.recordings)?;
        answers.retain(|recording| recording.argv().answers(asked));
        let key = answers.first().map(Recording::name).ok_or_else(|| GoldenFailure::Unanswered(asked.clone()))?;
        let claims = std::iter::successors(Some(Claim::first(key)), |claim| Some(claim.following()));
        let taken = |claim: Claim| claim.taken(&self.trail, asked).map(|taken| taken.then_some(claim)).transpose();
        let claimed = claims.take(answers.len()).find_map(taken).transpose()?;
        let exhausted = || GoldenFailure::Exhausted { asked: asked.clone(), answers_len: answers.len() };
        claimed.and_then(|claim| claim.answer(&answers).cloned()).ok_or_else(exhausted)
    }
}
