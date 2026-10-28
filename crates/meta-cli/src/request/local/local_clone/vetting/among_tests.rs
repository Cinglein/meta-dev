use super::context::Context;
use super::fake_host::FakeHost;
use super::vetted::Vetted;

const LIST: &str = "list";
const LISTING: &str = "worktree list";
const ON_BRANCH: &str = "symbolic-ref";
const OFF_BRANCH: &str = "HEAD is not on the recorded branch";
const SHARED: &str = "branch checked out elsewhere";
const UNLISTED: &str = "cannot inspect: offline";
const LISTINGS: [&str; 5] = [
    "worktree /trees/task-00000000000000b1\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n",
    "",
    "worktree /trees/task-00000000000000b1\nHEAD c1\ndetached\n",
    "worktree /trees/task-00000000000000b1\nHEAD c1\nbranch refs/heads/other\n",
    "worktree /trees/task-00000000000000b1\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n\nworktree /clone\nHEAD c1\nbranch refs/heads/task-00000000000000b1\n",
];
const VERDICTS: [&str; 5] = ["", OFF_BRANCH, OFF_BRANCH, OFF_BRANCH, SHARED];
const VERIFY: &str = "rev-parse";
const ONE_LISTING_NO_HEAD_READ_ONE_VERIFY_A_VERDICT: [usize; 3] = [1, 0, 2];

struct Sweep<'host>(&'host FakeHost, Context);

impl<'host> Sweep<'host> {
    fn new(host: &'host FakeHost) -> Self {
        Self(host, host.context())
    }

    fn verdict(&self) -> String {
        let vetted = Vetted::proved_among(&FakeHost::worktree(), &self.1, self.0, self.1.checkouts(self.0));
        vetted.err().map(|kept| kept.to_string()).unwrap_or_default()
    }

    fn twice(&self) -> [String; 2] {
        [self.verdict(), self.verdict()]
    }

    fn reads(&self) -> [usize; 3] {
        let calls = self.0.calls();
        [LISTING, ON_BRANCH, VERIFY].map(|word| calls.iter().filter(|call| call.contains(word)).count())
    }
}

#[test]
fn a_sweeps_one_listing_judges_head_and_sharing_without_a_head_read_and_a_failed_listing_keeps_the_worktree() {
    let hosts = LISTINGS.map(|listing| {
        let mut host = FakeHost::clean();
        host.answer_is(LIST, listing);
        host
    });
    let sweeps = hosts.each_ref().map(Sweep::new);
    assert_eq!(sweeps.each_ref().map(Sweep::twice), VERDICTS.map(|shown| [shown; 2]));
    assert_eq!(sweeps.each_ref().map(Sweep::reads), [ONE_LISTING_NO_HEAD_READ_ONE_VERIFY_A_VERDICT; 5]);
    let mut unlisted = FakeHost::clean();
    unlisted.fails(LIST);
    assert_eq!(Sweep::new(&unlisted).twice(), [UNLISTED; 2]);
}
