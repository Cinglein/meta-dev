use std::fmt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Fixture(git2::Repository);

pub struct Moment(i64);

impl Moment {
    pub const fn after_epoch(seconds: i64) -> Moment {
        Moment(seconds)
    }
}

impl Fixture {
    const NAME: &'static str = "t";
    const EMAIL: &'static str = "t@meta.invalid";
    const ORIGIN: &'static str = "origin";
    const HEAD: &'static str = "HEAD";
    const QUIET: [&'static str; 2] = ["maintenance.auto", "receive.autogc"];
    const NO_REFLOG: &'static str = "core.logAllRefUpdates";
    const EVERY_HEAD: [&'static str; 0] = [];
    const LINKED: &'static str = "worktrees";
    const LINK: &'static str = ".git";
    const WIP: &'static str = "WIP";
    const ORIG_HEAD: &'static str = "ORIG_HEAD";
    const SEED: &'static str = "seed";

    pub fn bare(dir: &Path, head: &str) -> Fixture {
        Self::laid(dir, head, true)
    }

    pub fn working(dir: &Path, origin: &Path, head: &str) -> Fixture {
        let fixture = Self::laid(dir, head, false);
        fixture.0.remote(Self::ORIGIN, origin.to_str().unwrap()).unwrap();
        fixture
    }

    pub fn opened(dir: &Path) -> Fixture {
        Fixture(git2::Repository::open(dir).unwrap())
    }

    fn laid(dir: &Path, head: &str, bare: bool) -> Fixture {
        let mut options = git2::RepositoryInitOptions::new();
        options.bare(bare).initial_head(head).external_template(false);
        let repository = git2::Repository::init_opts(dir, &options).unwrap();
        let mut config = repository.config().unwrap();
        Self::QUIET.iter().try_for_each(|key| config.set_bool(key, false)).unwrap();
        Fixture(repository)
    }

    pub fn pointed(&self, url: &Path) {
        self.0.remote_set_url(Self::ORIGIN, url.to_str().unwrap()).unwrap();
    }

    pub fn configured(&self, key: &str, value: &str) {
        self.0.config().unwrap().set_str(key, value).unwrap();
    }

    pub fn quietly_tracking(self, branch: &str) -> Fixture {
        self.0.config().unwrap().set_bool(Self::NO_REFLOG, false).unwrap();
        self.0.find_remote(Self::ORIGIN).unwrap().fetch(&Self::EVERY_HEAD, None, None).unwrap();
        {
            let tracked = [Self::ORIGIN, branch].join("/");
            let tip = self.0.find_branch(&tracked, git2::BranchType::Remote).unwrap().get().peel_to_commit().unwrap();
            self.0.branch(branch, &tip, false).unwrap().set_upstream(Some(&tracked)).unwrap();
            self.0.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();
        }
        self
    }

    pub fn committed(&self, message: &str) -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().cast_signed();
        self.committed_at(message, &Moment::after_epoch(now))
    }

    pub fn committed_at(&self, message: &str, at: &Moment) -> String {
        let parent = self.0.head().and_then(|head| head.peel_to_commit()).ok();
        let tree = parent
            .as_ref()
            .map_or_else(|| self.0.find_tree(self.0.treebuilder(None).unwrap().write().unwrap()), git2::Commit::tree);
        self.commit_on(Self::HEAD, &tree.unwrap(), message, at).to_string()
    }

    pub fn written(&self, reference: &str, name: &str, text: &str, at: &Moment) -> git2::Oid {
        let parent = self.0.find_reference(reference).and_then(|tip| tip.peel_to_tree()).ok();
        let mut tree = self.0.treebuilder(parent.as_ref()).unwrap();
        tree.insert(name, self.0.blob(text.as_bytes()).unwrap(), git2::FileMode::Blob.into()).unwrap();
        self.commit_on(reference, &self.0.find_tree(tree.write().unwrap()).unwrap(), Self::SEED, at)
    }

    fn commit_on(&self, reference: &str, tree: &git2::Tree<'_>, message: &str, at: &Moment) -> git2::Oid {
        let signature = git2::Signature::new(Self::NAME, Self::EMAIL, &git2::Time::new(at.0, 0)).unwrap();
        let parent = self.0.find_reference(reference).and_then(|tip| tip.peel_to_commit()).ok();
        let made = self.0.commit(Some(reference), &signature, &signature, message, tree, &Vec::from_iter(&parent));
        let made = made.unwrap();
        let resolved = self.0.find_reference(reference).unwrap().resolve().unwrap().name().unwrap().to_owned();
        self.relogged(reference, made, &signature);
        self.relogged(&resolved, made, &signature);
        made
    }

    pub fn branched(&self, prefix: &str, names: &[&str], to: git2::Oid) {
        let moved = |name: &&str| self.0.reference(&[prefix, name].concat(), to, true, Self::HEAD).map(drop);
        names.iter().try_for_each(moved).unwrap();
    }

    pub fn worktrees(&self, root: &Path, branches: &[&str], detached: &str) {
        let added = branches.iter().try_for_each(|branch| self.added(&root.join(branch), Some(branch)));
        added.and_then(|()| self.added(&root.join(detached), None)).unwrap();
    }

    fn added(&self, path: &Path, branch: Option<&str>) -> Result<(), git2::Error> {
        let name = path.file_name().unwrap().to_str().unwrap();
        let reference = branch.map(|branch| self.0.find_branch(branch, git2::BranchType::Local)).transpose()?;
        let reference = reference.map(git2::Branch::into_reference);
        self.0.worktree(name, path, Some(git2::WorktreeAddOptions::new().reference(reference.as_ref())))?;
        let admin = std::fs::canonicalize(self.0.path().join(Self::LINKED).join(name)).unwrap();
        std::fs::write(path.join(Self::LINK), format!("gitdir: {}\n", admin.display())).unwrap();
        let detached = || {
            let tree = Fixture::opened(path);
            tree.0.set_head_detached(tree.0.head()?.target().unwrap())?;
            self.0.find_branch(name, git2::BranchType::Local)?.delete()
        };
        branch.map_or_else(detached, |_| Ok(()))
    }

    pub fn rewound(&self) {
        let head = self.0.head().unwrap().peel_to_commit().unwrap();
        self.0.reference(Self::WIP, head.id(), true, Self::HEAD).unwrap();
        self.0.reset(head.parent(0).unwrap().as_object(), git2::ResetType::Hard, None).unwrap();
        self.0.find_reference(Self::ORIG_HEAD).into_iter().try_for_each(|mut orig| orig.delete()).unwrap();
    }

    pub fn has_branch(&self, name: &str) -> bool {
        self.0.find_branch(name, git2::BranchType::Local).is_ok()
    }

    fn relogged(&self, name: &str, made: git2::Oid, signature: &git2::Signature<'_>) {
        let mut log = self.0.reflog(name).unwrap();
        let newest = log.get(0).filter(|entry| entry.id_new() == made);
        let message = newest.and_then(|entry| entry.message().unwrap().map(str::to_owned));
        let rewritten = message.map_or(Ok(()), |message| {
            log.remove(0, false)?;
            log.append(made, signature, Some(&message))?;
            log.write()
        });
        rewritten.unwrap();
    }

    pub fn pushed(&self, to: &str) {
        let from = self.0.head().unwrap().name().unwrap().to_owned();
        self.0.find_remote(Self::ORIGIN).unwrap().push(&[format!("+{from}:refs/heads/{to}")], None).unwrap();
    }
}

impl fmt::Display for Fixture {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.head().unwrap().target().unwrap())
    }
}
