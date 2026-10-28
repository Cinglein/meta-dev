use std::ffi::OsStr;
use std::path::Path;

use meta_schema::{Claim, Event, Id, NewTask, Record, Task};
use meta_state::State;

use super::{BranchRefs, ClaimPlan, ClaimStep, Holding, Undo};
use crate::request::local::local_settings::LocalSettings;

const AGENT: &str = "agent";
const NEWCOMER: &str = "newcomer";
const TAKEN: &str = "/work/clone-worktrees/taken";
const UNIGNORED: &str = "meta.local.toml is not gitignored";
const HEAD: &str = "origin/HEAD";

struct Refs(State, Id<Task>, [String; 3]);

impl Refs {
    fn new() -> Self {
        let opened = NewTask::new("features-and-maintenance".parse().unwrap(), AGENT.parse().unwrap(), vec![]);
        let task = opened.task();
        let state =
            State::fold(&[Record::new(AGENT.parse().unwrap(), Event::TaskOpened(opened)), Self::claim(AGENT, task)]);
        Self(state, task, [format!("refs/heads/{task}"), format!("origin/{task}"), HEAD.to_owned()])
    }

    fn claim(actor: &str, task: Id<Task>) -> Record {
        Record::new(actor.parse().unwrap(), Event::TaskClaimed(Claim::new(task, None)))
    }

    fn known(&self, known: &[&String]) -> BranchRefs {
        let verified = |name: &OsStr| known.iter().any(|known| OsStr::new(known.as_str()) == name).then_some(());
        BranchRefs::gathered("origin".parse().unwrap(), (&self.1).into(), verified)
    }

    fn plan(&self, holding: Holding, known: &[&String]) -> Option<Vec<ClaimStep>> {
        ClaimPlan::new(holding, &self.known(known), None).ok().map(|plan| plan.steps().to_vec())
    }
}

#[test]
fn the_holder_resumes_a_newcomer_claims_afresh_and_the_base_is_the_local_then_remote_then_default_branch() {
    let refs = Refs::new();
    let [local, branch, head] = &refs.2;
    let holding = |actor: &str| Holding::vetted(&refs.0, &Refs::claim(actor, refs.1), refs.1).ok();
    assert_eq!([holding(AGENT), holding(NEWCOMER)], [Some(Holding::Held), Some(Holding::Fresh)]);
    let base = |holding, known: &[&String]| refs.known(known).base(holding).ok();
    let bases = [base(Holding::Held, &[local]), base(Holding::Fresh, &[local]), base(Holding::Fresh, &[branch, head])];
    let [branched, headed] = [branch, head].map(|name| Some(name.parse().ok()));
    let defaulted = [base(Holding::Held, &[head]), base(Holding::Fresh, &[])];
    assert_eq!((bases, defaulted), ([Some(None), None, branched], [headed, None]));
}

#[test]
fn a_plan_branches_only_from_a_base_writes_only_a_fresh_claim_and_refuses_a_taken_path() {
    let refs = Refs::new();
    let [local, branch, _] = &refs.2;
    let resumed = [ClaimStep::Pruned, ClaimStep::Configured, ClaimStep::Added, ClaimStep::Settled];
    let mut fresh = resumed.to_vec();
    fresh.insert(2, ClaimStep::Branched(branch.parse().unwrap()));
    fresh.push(ClaimStep::Written);
    let plans = [refs.plan(Holding::Held, &[local]), refs.plan(Holding::Fresh, &[branch])];
    assert_eq!(plans, [Some(resumed.to_vec()), Some(fresh)]);
    let taken = ClaimPlan::new(Holding::Fresh, &refs.known(&[branch]), Some(Path::new(TAKEN)));
    assert!(taken.is_err_and(|refusal| refusal.to_string().starts_with(TAKEN)));
}

#[test]
fn rollback_undoes_the_done_steps_last_first_and_an_unignored_local_settings_file_is_refused() {
    let refs = Refs::new();
    let fresh = refs.plan(Holding::Fresh, &[&refs.2[1]]).unwrap_or_default();
    let undone = [2, 4, 6].map(|done| ClaimPlan::rollback(fresh.get(..done).unwrap_or_default()));
    let [removed, deleted] = [Undo::WorktreeRemoved, Undo::BranchDeleted];
    assert_eq!(undone, [vec![], vec![removed, deleted], vec![Undo::Untrailed, removed, deleted]]);
    let admitted = [(None, None), (Some(()), Some(())), (Some(()), None)]
        .map(|(present, ignored)| LocalSettings::admitted(present, || ignored).map_err(|refusal| refusal.to_string()));
    let refused = admitted.map(|admitted| admitted.err().is_some_and(|why| why.starts_with(UNIGNORED)));
    assert_eq!(refused, [false, false, true]);
}
