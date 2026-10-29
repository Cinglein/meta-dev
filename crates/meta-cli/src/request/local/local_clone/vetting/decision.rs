use super::halt::Halt;
use super::kept::Kept;
use super::next::Next;
use super::replay::Replay;
use super::turn::Turn;

// A pure step function: given every turn so far, say what comes next.
pub trait Decision {
    type Answer;

    fn next(&self, journal: &[Turn]) -> Next<Self::Answer>;
}

// Any function that reads the world through a Replay is a Decision: it is re-run from the start
// over the journal at each turn, and its first unanswered read or write is the next step.
#[derive(Debug)]
pub struct Replayed<Judge>(Judge);

impl<Judge> Replayed<Judge> {
    pub fn new<Answer>(judge: Judge) -> Self
    where
        Judge: Fn(&Replay<'_>) -> Result<Answer, Halt>,
    {
        Self(judge)
    }
}

impl<Answer, Judge: Fn(&Replay<'_>) -> Result<Answer, Halt>> Decision for Replayed<Judge> {
    type Answer = Result<Answer, Kept>;

    fn next(&self, journal: &[Turn]) -> Next<Self::Answer> {
        match (self.0)(&Replay::new(journal)) {
            Ok(answer) => Next::Done(Ok(answer)),
            Err(Halt::Kept(kept)) => Next::Done(Err(kept)),
            Err(Halt::Look(looks)) => Next::Look(looks),
            Err(Halt::Act(changes)) => Next::Act(changes),
        }
    }
}
