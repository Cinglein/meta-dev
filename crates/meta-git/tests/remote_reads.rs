#![allow(clippy::unwrap_used)]

use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use meta_git::{
    FastForward, Fetch, FileCommit, Files, Identity, Layout, ObjectId, RefMove, Remote, Repository, Transport, Writes,
};
use meta_run::Budget;

const ORIGIN: &str = "origin";
const ALIASED: &str = "aliased";
const BARE: &str = "o.git";
const NAMED: &str = "o";
const CLONE: &str = "clone";
const SHORT: &str = "short:";
const HEADS: [&str; 3] = ["refs/heads/alias", "refs/heads/main", "refs/heads/side"];
const LOCAL: [&str; 3] = ["/srv/o", "o", "./a:b"];
const COMMAND: [&str; 4] = ["file:///srv/o", "host:o", "ssh://host/o", ""];
const PATIENT: Duration = Duration::from_secs(30);
const REWRITE: [&str; 2] = ["url.", ".insteadOf"];
const REFSPEC: &str = "+refs/heads/*:refs/remotes/origin/*";
const RECEIVE_PACK: &str = "remote.origin.receivepack";
const RECEIVE_PACK_PROGRAM: &str = "git-receive-pack";

struct Origin {
    root: PathBuf,
    repository: Repository,
    first: ObjectId,
    second: ObjectId,
}

impl Origin {
    fn made(root: &Path) -> Origin {
        let repository = Repository::created(&root.join(BARE), Layout::Bare).unwrap();
        let commit = |parent: Option<ObjectId>| {
            let files: Files = [(ORIGIN.to_owned(), Some(ORIGIN.to_owned()))].into_iter().collect();
            let author = Identity::new(ORIGIN.to_owned(), ORIGIN.to_owned());
            repository.write(&FileCommit::new(parent, files, author, ORIGIN.to_owned())).unwrap()
        };
        let first = commit(None);
        let second = commit(Some(first));
        let [alias, main, side] = HEADS;
        let moved = |name: &str, to| repository.write(&RefMove::new(name.parse().unwrap(), None, to)).unwrap();
        moved(main, first);
        moved(side, second);
        git2::Repository::open(root.join(BARE)).unwrap().reference_symbolic(alias, main, true, alias).unwrap();
        let clone = git2::Repository::init(root.join(CLONE)).unwrap();
        let named = root.join(NAMED);
        clone.remote(ORIGIN, named.to_str().unwrap()).unwrap();
        clone.remote(ALIASED, SHORT).unwrap();
        let [before, after] = REWRITE;
        clone.config().unwrap().set_str(&[before, named.to_str().unwrap(), after].concat(), SHORT).unwrap();
        Origin { root: root.to_path_buf(), repository, first, second }
    }

    fn remote(&self, name: &str) -> Remote {
        Remote::new(self.root.join(CLONE), name.parse().unwrap(), Budget::new(PATIENT))
    }

    fn listed(&self, remote: &Remote, side: ObjectId) -> bool {
        let shown = remote.heads().unwrap().unwrap().to_string();
        shown == Listed { main: self.first, side }.to_string()
    }
}

struct Listed {
    main: ObjectId,
    side: ObjectId,
}

impl fmt::Display for Listed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [alias, main, side] = HEADS;
        [(alias, self.main), (main, self.main), (side, self.side)]
            .iter()
            .try_for_each(|(name, id)| writeln!(formatter, "{id}\t{name}"))
    }
}

#[test]
fn a_url_is_local_only_as_a_plain_path_and_an_instead_of_rewrite_applies_first() {
    let root = tempfile::tempdir().unwrap();
    let origin = Origin::made(root.path());
    let read = |urls: &[&str]| urls.iter().map(|url| url.parse::<Transport>().unwrap()).collect::<Vec<_>>();
    assert_eq!(read(&LOCAL), LOCAL.map(|path| Transport::Local(PathBuf::from(path))));
    assert_eq!(read(&COMMAND), COMMAND.map(|_| Transport::Command));
    assert_eq!(origin.remote(ALIASED).transport(), Transport::Local(root.path().join(NAMED)));
}

#[test]
fn local_heads_list_like_ls_remote_heads_through_a_dot_git_suffix_once_until_forgotten() {
    let root = tempfile::tempdir().unwrap();
    let origin = Origin::made(root.path());
    let remote = origin.remote(ORIGIN);
    assert!(origin.listed(&remote, origin.second));
    let [_, _, side] = HEADS;
    origin.repository.write(&RefMove::new(side.parse().unwrap(), Some(origin.second), origin.first)).unwrap();
    assert!(origin.listed(&remote, origin.second));
    remote.forget_heads();
    assert!(origin.listed(&remote, origin.first));
}

#[test]
fn a_plain_path_remote_takes_fetches_and_pushes_in_process_unless_a_helper_program_is_configured() {
    let root = tempfile::tempdir().unwrap();
    let origin = Origin::made(root.path());
    let remote = origin.remote(ORIGIN);
    let fetch = Fetch::from(REFSPEC.to_owned());
    let [_, main, _] = HEADS;
    let push = FastForward::new(origin.first, main.parse().unwrap());
    assert!(remote.local_end(&fetch).is_some() && remote.local_end(&push).is_some());
    let clone = git2::Repository::open(root.path().join(CLONE)).unwrap();
    clone.config().unwrap().set_str(RECEIVE_PACK, RECEIVE_PACK_PROGRAM).unwrap();
    assert!(remote.local_end(&fetch).is_some() && remote.local_end(&push).is_none());
}
