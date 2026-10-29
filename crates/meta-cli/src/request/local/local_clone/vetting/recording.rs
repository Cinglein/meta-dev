use std::path::PathBuf;

use super::alternates::Alternates;
use super::cause::Cause;
use super::change::Change;
use super::context::Context;
use super::decision::Decision;
use super::entry_kind::EntryKind;
use super::found::Found;
use super::host::Host;
use super::interpreter::Interpreter;
use super::kept::Kept;
use super::look::Look;
use super::made::Made;
use super::next::Next;
use super::occupancy::Occupancy;
use super::scene::Scene;
use super::seen::Seen;
use super::shared_node::SharedNode;
use super::turn::Turn;
use crate::request::local::byte_count::ByteCount;

// The views a real run on a Scene returned, replayed into a pure decision. A test changes one view
// (the hazard it is about) and nothing else; a read the run never made gets the empty answer.
#[derive(Clone, Debug)]
pub struct Recording<'scene> {
    scene: &'scene Scene,
    context: Context,
    answers: Vec<(Look, Seen)>,
    changed: Vec<(Look, Seen)>,
    words: Vec<(&'static str, Result<String, Kept>)>,
    walked: Vec<Found>,
    open: Vec<PathBuf>,
}

impl<'scene> Recording<'scene> {
    const TURNS: usize = 256;
    const UNRECORDED: &'static str = "unrecorded git call";
    const OFFLINE: &'static str = "offline";

    // Run the decision for real through the interpreter and keep every view it was given.
    pub fn of<D: Decision>(scene: &'scene Scene, context: Context, decision: impl Fn(&Context) -> D) -> Self {
        let host = Host::default();
        let interpreter = Interpreter::new(&host);
        let mut journal = Vec::new();
        let run = decision(&context);
        let _answer = (0..Self::TURNS).find_map(|_| interpreter.turned(&run, &mut journal));
        let answers = journal
            .into_iter()
            .flat_map(|turn| match turn {
                Turn::Looked(looks, seen) => looks.into_iter().zip(seen).collect::<Vec<_>>(),
                Turn::Acted(..) => Vec::new(),
            })
            .collect();
        Self { scene, context, answers, changed: Vec::new(), words: Vec::new(), walked: Vec::new(), open: Vec::new() }
    }

    pub fn empty(scene: &'scene Scene, context: Context) -> Self {
        Self { scene, context, answers: Vec::new(), changed: Vec::new(), words: Vec::new(), walked: Vec::new(), open: Vec::new() }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    // Replay a decision on the recorded views; changes succeed without touching anything.
    pub fn answered<D: Decision>(&self, decision: &D) -> (Option<D::Answer>, Vec<Turn>) {
        let mut journal = Vec::new();
        for _ in 0..Self::TURNS {
            match decision.next(&journal) {
                Next::Done(answer) => return (Some(answer), journal),
                Next::Look(looks) => {
                    let seen = looks.iter().map(|look| self.seen(look)).collect();
                    journal.push(Turn::Looked(looks, seen));
                }
                Next::Act(changes) => {
                    let made = changes.iter().map(Self::made).collect();
                    journal.push(Turn::Acted(changes, made));
                }
            }
        }
        (None, journal)
    }

    pub fn entry_is(&mut self, place: &str, kind: EntryKind) {
        self.changed.push((Look::Entry(self.scene.real(place)), Seen::Entry(kind)));
    }

    pub fn text_is(&mut self, place: &str, text: &str) {
        self.changed.push((Look::Text(self.scene.real(place)), Seen::Text(Some(self.scene.translated(text)))));
    }

    pub fn moved(&mut self, place: &str, to: &str) {
        self.changed.push((Look::Canonical(self.scene.real(place)), Seen::Canonical(Some(self.scene.real(to)))));
    }

    pub fn answer_is(&mut self, word: &'static str, answer: &str) {
        self.words.insert(0, (word, Ok(self.scene.translated(answer))));
    }

    pub fn fails(&mut self, word: &'static str) {
        self.words.insert(0, (word, Err(Kept::Uninspected(Cause::new(Self::OFFLINE)))));
    }

    pub fn walks(&mut self, paths: &[&str], kind: EntryKind) {
        self.walked.extend(paths.iter().map(|path| Found::new(PathBuf::from(path), kind, ByteCount::default())));
    }

    pub fn links(&mut self, paths: &[&str], node: SharedNode) {
        let linked = paths.iter().map(|path| Found::new(PathBuf::from(path), EntryKind::File, ByteCount::default()));
        self.walked.extend(linked.map(|found| found.sharing(node)));
    }

    pub fn placed(&mut self, places: [&str; 3]) {
        let [root, checkout, common] = places.map(|place| self.scene.real(place));
        self.context = Context::new(checkout, common, root, "origin".parse().unwrap()).lent(self.context.lenders().to_vec());
    }

    pub fn lent(&mut self, objects: &str, listed: &str) {
        let lender = Alternates::new(self.scene.real(objects), listed);
        self.context = self.context.clone().lent([self.context.lenders(), &[lender]].concat());
    }

    pub fn looks(journal: &[Turn]) -> Vec<Look> {
        journal
            .iter()
            .flat_map(|turn| match turn {
                Turn::Looked(looks, _) => looks.clone(),
                Turn::Acted(..) => Vec::new(),
            })
            .collect()
    }

    fn seen(&self, look: &Look) -> Seen {
        let changed = self.changed.iter().rev().find(|(changed, _)| changed == look).map(|(_, seen)| seen.clone());
        changed.unwrap_or_else(|| match look {
            Look::Git(call) => {
                let word = self.words.iter().find(|(word, _)| call.words().iter().any(|argument| argument == word));
                word.map(|(_, answer)| Seen::Git(answer.clone())).unwrap_or_else(|| self.recorded_git(look))
            }
            Look::Walk(path) if *path == self.scene.real(Scene::PATH) => match self.recorded(look) {
                Some(Seen::Walk(found)) => Seen::Walk([found, self.walked.clone()].concat()),
                _ => Seen::Walk(self.walked.clone()),
            },
            Look::Occupancy(_) => Seen::Occupancy(Occupancy::Listed(self.open.clone())),
            Look::Entry(_) => self.recorded(look).unwrap_or(Seen::Entry(EntryKind::Missing)),
            Look::Text(_) => self.recorded(look).unwrap_or(Seen::Text(None)),
            Look::Canonical(path) => self.recorded(look).unwrap_or_else(|| Seen::Canonical(Some(Alternates::resolved(path)))),
            Look::Walk(_) => self.recorded(look).unwrap_or(Seen::Walk(Vec::new())),
            Look::Lenders(_) => self.recorded(look).unwrap_or(Seen::Lenders(Vec::new())),
        })
    }

    fn recorded(&self, look: &Look) -> Option<Seen> {
        self.answers.iter().find(|(asked, _)| asked == look).map(|(_, seen)| seen.clone())
    }

    // A git read the run made with other operands (say rev-list after the remote changed) gets the
    // answer the run got for the same subcommand.
    fn recorded_git(&self, look: &Look) -> Seen {
        let Look::Git(call) = look else { return Seen::Git(Err(Kept::Uninspected(Cause::new(Self::UNRECORDED)))) };
        self.recorded(look).unwrap_or_else(|| {
            let same = self.answers.iter().find(|(asked, _)| match asked {
                Look::Git(asked) => asked.words().first() == call.words().first(),
                _ => false,
            });
            same.map_or(Seen::Git(Err(Kept::Uninspected(Cause::new(Self::UNRECORDED)))), |(_, seen)| seen.clone())
        })
    }

    fn made(change: &Change) -> Made {
        match change {
            Change::Git(_) | Change::Destroy(_) => Made::Git(Ok(String::new())),
            Change::Freeze(_, plan) => Made::Frozen(Ok(plan.clone())),
            Change::Thaw(_) => Made::Thawed,
        }
    }
}
