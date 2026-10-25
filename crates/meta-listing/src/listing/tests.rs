use meta_schema::{Actor, Claim, Entry, EntryNote, Event, NewTask, Record, Text};

use meta_state::{State, Summary};

use crate::kind::Kind;
use crate::listing::Listing;
use crate::scope::Scope;

const OWNER: &str = "owner";
const ORIGINAL: &str = "wrong id task-3afc044494f2045f";
const CORRECTED: &str = "right id task-0351e1866c684d95";
const REASON: &str = "owner kept early stop";
const RETIRED_SUMMARY: &str = "Tasks: 0 unfinished, 0 done, 1 retired";

struct Script;

impl Script {
    fn text(text: &str) -> Text {
        text.parse().unwrap()
    }

    fn record(event: Event) -> Record {
        Record::new(OWNER.parse::<Actor>().unwrap(), event)
    }

    fn retired_and_corrected(task: &NewTask) -> Vec<Event> {
        let entry = Entry::Task(task.task());
        vec![
            Event::TaskOpened(task.clone()),
            Event::EntryCorrected(EntryNote::new(entry, Self::text(CORRECTED))),
            Event::EntryRetired(EntryNote::new(entry, Self::text(REASON))),
        ]
    }
}

#[test]
fn a_retired_task_is_kept_corrected_and_closed_to_further_work() {
    let task = NewTask::new(Script::text(ORIGINAL), vec![]);
    let records: Vec<Record> = Script::retired_and_corrected(&task).into_iter().map(Script::record).collect();
    let state = State::fold(&records);
    let listed = Listing::new(&state, Kind::Tasks, Scope::All).to_string();
    assert!(listed.contains(CORRECTED) && listed.contains(REASON) && !listed.contains(ORIGINAL));
    assert!(Summary::new(&state).to_string().starts_with(RETIRED_SUMMARY));
    assert!(state.vet(&Script::record(Event::TaskClaimed(Claim::new(task.task(), None)))).is_err());
}
