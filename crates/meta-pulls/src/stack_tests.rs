use std::path::Path;

use meta_golden::Golden;
use meta_run::{CommandLine, Port, PullArgv};
use meta_schema::PullRequest;

use crate::live_pull::{HeadCommit, MergeState};
use crate::{Action, GhRefusal, MergeStatus, Stacked, Stacks, Stop};

const RECORDED: &str = include_str!("../../../goldens/queue/stack.json");
const SHA: &str = "6fc5e0e2f38a1a24e8572a890868e92f7dc1d99f";
const UUID: &str = "630b9d5e-3f2a-4f7e-8b0c-2d5f9a8c1e42";
const TOP: &str = "244";
const BELOW: &str = "242";
const OUTSIDE: &str = "227";
const REPLIES: [&str; 4] = ["pending", "merged Pull request is already merged.", "enqueued", "failed Merge conflict"];

struct Recorded;

impl Recorded {
    fn action() -> Action {
        let top: PullRequest = TOP.parse().unwrap();
        Action::StackMerge(Stacked::new(
            top,
            HeadCommit::try_from(SHA.to_owned()).unwrap(),
            vec![BELOW.parse().unwrap()],
        ))
    }

    fn line(call: PullArgv, operands: Vec<std::ffi::OsString>) -> String {
        CommandLine::from(&Port::<PullArgv>::new(Path::new("")).run(call, operands)).to_string()
    }
}

#[test]
fn a_recorded_stack_reply_lists_its_base_and_its_members_bottom_first_with_their_heads() {
    let stacks: Stacks = serde_json::from_str(RECORDED).unwrap();
    let stack = stacks.clone().containing(TOP.parse().unwrap()).unwrap();
    let members = stack.through(TOP.parse().unwrap());
    let listed = members.iter().map(|member| format!("{} {}", member.number(), member.head())).collect::<Vec<_>>();
    Golden::for_test(module_path!(), "members").debugged(&(stack.base().to_string(), listed)).unwrap();
    assert!(stacks.containing(OUTSIDE.parse().unwrap()).is_none());
}

#[test]
fn the_stack_merge_puts_the_tested_head_to_the_async_endpoint_and_reads_its_result_back_by_id() {
    let action = Recorded::action();
    let put = action.call().map(|call| Recorded::line(call, action.operands()));
    let get = action.awaiting(UUID).map(|operands| Recorded::line(PullArgv::MergeResult, operands));
    let statuses = REPLIES.map(|reply| format!("{:?}", reply.parse::<MergeStatus>()));
    Golden::for_test(module_path!(), "argv").debugged(&(put, get, statuses.to_vec())).unwrap();
}

#[test]
fn a_stack_stop_names_the_requested_pull_request_and_the_member_that_blocks_it() {
    let [top, below]: [PullRequest; 2] = [TOP, BELOW].map(|number| number.parse().unwrap());
    let head = |sha: &str| HeadCommit::try_from(sha.to_owned()).unwrap();
    let blocked = Stop::NotMergeable { pull_request: below, state: MergeState::Behind };
    let stops = [
        Stop::Downstack { requested: top, blocker: Box::new(blocked) },
        Stop::Downstack { requested: top, blocker: Box::new(Stop::Closed(below)) },
        Stop::Moved { pull_request: below, listed: head(SHA), read: head(UUID) },
        Stop::Refused { action: Recorded::action(), refusal: GhRefusal::new("HTTP 422") },
    ];
    Golden::for_test(module_path!(), "stops").debugged(&stops.map(|stop| stop.to_string())).unwrap();
}
