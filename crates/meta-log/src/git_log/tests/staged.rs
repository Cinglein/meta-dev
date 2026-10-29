use std::cell::Cell;

use meta_git::{FileCommit, Identity, RefMove, Repository, Writes};

use super::{BRANCH, FIRST, Fixture, PAIR, Port, SINGLE};
use crate::LogStore;
use crate::git_log::GitLog;
use crate::log_failure::LogFailure;

#[test]
fn a_staged_ref_moved_while_the_writer_decided_races_and_the_writer_decides_again() {
    let fixture = Fixture::new(SINGLE);
    let log = fixture.log(FIRST);
    log.append::<LogFailure>(&mut |_| Ok(Fixture::record())).unwrap();
    let clone = fixture.clones[FIRST].path();
    let decisions = Cell::new(0);
    log.append::<LogFailure>(&mut |_| {
        decisions.set(decisions.get() + 1);
        let repository = Repository::open(clone).unwrap();
        let author = Identity::new(BRANCH.to_owned(), BRANCH.to_owned());
        let other =
            repository.write(&FileCommit::new(None, [].into_iter().collect(), author, BRANCH.to_owned())).unwrap();
        let staged = Port::target(clone, GitLog::STAGED);
        (decisions.get() == SINGLE)
            .then(|| repository.write(&RefMove::new(GitLog::STAGED.parse().unwrap(), staged, other)).unwrap());
        Ok(Fixture::record())
    })
    .unwrap();
    assert_eq!(
        (decisions.get(), Port::target(clone, GitLog::STAGED)),
        (PAIR, Port::target(fixture.origin.path(), BRANCH))
    );
}
