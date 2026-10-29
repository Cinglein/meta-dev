use std::num::NonZeroUsize;

use super::cause::Cause;
use super::change::Change;
use super::decision::Decision;
use super::host::Host;
use super::kept::Kept;
use super::look::Look;
use super::made::Made;
use super::next::Next;
use super::seen::Seen;
use super::turn::Turn;

// The one runner of decisions. It keeps no views between turns: a Look always goes to the world.
// It runs many decisions in lockstep and sends all their Looks of one round as one batch, with
// duplicate reads in that batch made once. A Destroy runs only as the first change of an Act that
// comes straight after a Look; otherwise it is refused unrun, and so is every change after it.
#[derive(Debug)]
pub struct Interpreter<'host> {
    host: &'host Host,
}

impl<'host> Interpreter<'host> {
    const TURNS: usize = 256;
    const STALLED: &'static str = "decision did not finish";

    pub fn new(host: &'host Host) -> Self {
        Self { host }
    }

    pub fn ran<D: Decision>(&self, decision: &D) -> Result<D::Answer, Kept> {
        let mut journal = Vec::new();
        (0..Self::TURNS)
            .find_map(|_| self.turned(decision, &mut journal))
            .ok_or_else(|| Kept::Uninspected(Cause::new(Self::STALLED)))
    }

    pub fn ran_all<D: Decision>(&self, decisions: &[D]) -> Vec<Result<D::Answer, Kept>> {
        let mut journals: Vec<Vec<Turn>> = decisions.iter().map(|_| Vec::new()).collect();
        let mut answers: Vec<Option<D::Answer>> = decisions.iter().map(|_| None).collect();
        for _ in 0..Self::TURNS {
            let open: Vec<usize> = (0..decisions.len()).filter(|at| answers.get(*at).is_some_and(Option::is_none)).collect();
            if open.is_empty() {
                break;
            }
            let mut looking: Vec<(usize, Vec<Look>)> = Vec::new();
            for at in open {
                let (Some(decision), Some(journal), Some(answer)) =
                    (decisions.get(at), journals.get_mut(at), answers.get_mut(at))
                else {
                    continue;
                };
                match decision.next(journal) {
                    Next::Done(done) => *answer = Some(done),
                    Next::Act(changes) => self.act(journal, changes),
                    Next::Look(looks) => looking.push((at, looks)),
                }
            }
            let batch: Vec<Look> = looking.iter().flat_map(|(_, looks)| looks.iter().cloned()).collect();
            let mut seen = self.batched(&batch).into_iter();
            for (at, looks) in looking {
                let answered: Vec<Seen> = seen.by_ref().take(looks.len()).collect();
                if let Some(journal) = journals.get_mut(at) {
                    journal.push(Turn::Looked(looks, answered));
                }
            }
        }
        answers.into_iter().map(|answer| answer.ok_or_else(|| Kept::Uninspected(Cause::new(Self::STALLED)))).collect()
    }

    // One turn: run what the decision asks next and append it, or return its answer.
    pub fn turned<D: Decision>(&self, decision: &D, journal: &mut Vec<Turn>) -> Option<D::Answer> {
        match decision.next(journal) {
            Next::Done(answer) => Some(answer),
            Next::Act(changes) => {
                self.act(journal, changes);
                None
            }
            Next::Look(looks) => {
                let seen = self.batched(&looks);
                journal.push(Turn::Looked(looks, seen));
                None
            }
        }
    }

    fn act(&self, journal: &mut Vec<Turn>, changes: Vec<Change>) {
        let fresh = matches!(journal.last(), Some(Turn::Looked(..)));
        let mut refused = false;
        let made = changes
            .iter()
            .enumerate()
            .map(|(at, change)| {
                refused = refused || (change.destroys() && !(fresh && at == 0));
                if refused { Made::Git(Err(Kept::Stale)) } else { self.host.made(change) }
            })
            .collect();
        journal.push(Turn::Acted(changes, made));
    }

    fn batched(&self, looks: &[Look]) -> Vec<Seen> {
        let mut unique: Vec<&Look> = Vec::new();
        looks.iter().for_each(|look| {
            if !unique.contains(&look) {
                unique.push(look);
            }
        });
        let width = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
        let seen: Vec<Seen> = unique
            .chunks(width)
            .flat_map(|chunk| {
                std::thread::scope(|scope| {
                    let running: Vec<_> = chunk.iter().map(|look| scope.spawn(|| self.host.seen(look))).collect();
                    running
                        .into_iter()
                        .map(|run| run.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        looks
            .iter()
            .filter_map(|look| unique.iter().position(|known| *known == look).and_then(|at| seen.get(at)).cloned())
            .collect()
    }
}
