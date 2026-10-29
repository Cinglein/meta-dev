use super::change::Change;
use super::kept::Kept;
use super::look::Look;

// Why a replayed decision stopped: it reached a verdict to keep the worktree, or it needs the world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Halt {
    Kept(Kept),
    Look(Vec<Look>),
    Act(Vec<Change>),
}

impl Halt {
    // Split a verdict from a pending step: a Kept becomes a value, a Look or Act still halts.
    pub fn settled<Value>(judged: Result<Value, Self>) -> Result<Result<Value, Kept>, Self> {
        match judged {
            Ok(value) => Ok(Ok(value)),
            Err(Self::Kept(kept)) => Ok(Err(kept)),
            Err(pending) => Err(pending),
        }
    }

}

impl From<Kept> for Halt {
    fn from(kept: Kept) -> Self {
        Self::Kept(kept)
    }
}
