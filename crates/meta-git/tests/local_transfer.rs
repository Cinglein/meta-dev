#![allow(clippy::unwrap_used)]

use std::path::Path;

use meta_git::{FastForward, Fetch, RemoteChange};

const BRANCH: &str = "refs/heads/meta/log";
const OLD: &str = "refs/heads/meta/old";
const LOCK: &str = "refs/heads/meta/log.lock";
const THERE: &str = "there";
const HERE: &str = "here";
const MESSAGES: [&str; 4] = ["first", "second", "sibling", "third"];
const GUARDS: [&str; 2] = ["hooks/update", "hooks/pre-receive.sample"];
const RECEIVE: &str = "receive.denyDeletes";
const REFSPEC: &str = "+refs/heads/meta/*:refs/remotes/origin/meta/*";
const TRACKED: [&str; 3] = ["refs/remotes/origin/main", "refs/remotes/origin/meta/log", "refs/remotes/origin/meta/old"];

struct Ends {
    root: tempfile::TempDir,
    here: git2::Repository,
    there: git2::Repository,
}

impl Ends {
    fn made(bare: bool) -> Ends {
        let root = tempfile::tempdir().unwrap();
        let there = root.path().join(THERE);
        let there = if bare { git2::Repository::init_bare(there) } else { git2::Repository::init(there) }.unwrap();
        let here = git2::Repository::init(root.path().join(HERE)).unwrap();
        Ends { root, here, there }
    }

    fn committed(&self, parent: Option<git2::Oid>, message: &str) -> git2::Oid {
        let tree = self.here.find_tree(self.here.treebuilder(None).unwrap().write().unwrap()).unwrap();
        let signature = git2::Signature::now(message, message).unwrap();
        let parents: Vec<_> = parent.iter().map(|parent| self.here.find_commit(*parent).unwrap()).collect();
        self.here.commit(None, &signature, &signature, message, &tree, &parents.iter().collect::<Vec<_>>()).unwrap()
    }

    fn pushed(&self, id: git2::Oid) -> Result<(), git2::Error> {
        FastForward::new(id.into(), BRANCH.parse().unwrap()).moved(&self.here, &self.there)
    }

    fn served(path: &Path) -> Option<()> {
        let change = FastForward::new(git2::Oid::ZERO_SHA1.into(), BRANCH.parse().unwrap());
        change.in_process(git2::Repository::open(path).unwrap()).map(drop)
    }
}

#[test]
fn a_fast_forward_lands_and_a_non_fast_forward_or_a_held_git_lock_is_refused_leaving_the_tip() {
    let ends = Ends::made(true);
    let [first, second, sibling, third] = MESSAGES;
    let first = ends.committed(None, first);
    let [second, sibling] = [second, sibling].map(|message| ends.committed(Some(first), message));
    let third = ends.committed(Some(second), third);
    let tip = || ends.there.refname_to_id(BRANCH).ok();
    assert!(ends.pushed(first).is_ok() && ends.pushed(second).is_ok() && tip() == Some(second));
    std::fs::write(ends.there.path().join(LOCK), "").unwrap();
    assert!(ends.pushed(sibling).is_err() && ends.pushed(third).is_err() && tip() == Some(second));
}

#[test]
fn a_hook_a_receive_setting_or_the_checked_out_branch_of_a_working_remote_keeps_a_push_on_the_git_cli() {
    let [hooked, sampled, configured] = [true, true, true].map(Ends::made);
    let [hook, sample] = GUARDS;
    std::fs::create_dir_all(hooked.there.path().join(hook).parent().unwrap()).unwrap();
    std::fs::write(hooked.there.path().join(hook), "").unwrap();
    std::fs::create_dir_all(sampled.there.path().join(sample).parent().unwrap()).unwrap();
    std::fs::write(sampled.there.path().join(sample), "").unwrap();
    configured.there.config().unwrap().set_bool(RECEIVE, true).unwrap();
    let working = Ends::made(false);
    working.there.set_head(BRANCH).unwrap();
    let served = [&hooked, &sampled, &configured, &working].map(|ends| Ends::served(ends.there.path()));
    assert_eq!(served, [None, Some(()), None, None]);
}

#[test]
fn a_fetch_copies_meta_heads_and_prunes_one_the_remote_dropped_only_under_its_refspec() {
    let ends = Ends::made(true);
    let [first, ..] = MESSAGES;
    let first = ends.committed(None, first);
    ends.pushed(first).unwrap();
    ends.there.reference(OLD, first, true, OLD).unwrap();
    ends.here.reference(TRACKED[0], first, true, OLD).unwrap();
    let fetched = || Fetch::from(REFSPEC.to_owned()).moved(&ends.here, &ends.there);
    let listed = |name: &str| ends.here.refname_to_id(name).ok();
    fetched().unwrap();
    assert_eq!(TRACKED.map(listed), [Some(first), Some(first), Some(first)]);
    ends.there.find_reference(OLD).unwrap().delete().unwrap();
    fetched().unwrap();
    assert_eq!(TRACKED.map(listed), [Some(first), Some(first), None]);
    drop(ends.root);
}
