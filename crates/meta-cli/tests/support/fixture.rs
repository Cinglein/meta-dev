use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Fixture(git2::Repository);

impl Fixture {
    const NAME: &'static str = "t";
    const EMAIL: &'static str = "t@meta.invalid";
    const ORIGIN: &'static str = "origin";
    const HEAD: &'static str = "HEAD";
    const QUIET: [&'static str; 2] = ["maintenance.auto", "receive.autogc"];
    const NO_REFLOG: &'static str = "core.logAllRefUpdates";

    pub fn bare(dir: &Path, head: &str) -> Fixture {
        Self::laid(dir, head, true)
    }

    pub fn working(dir: &Path, head: &str) -> Fixture {
        Self::laid(dir, head, false)
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

    pub fn linked(self, origin: &Path) -> Fixture {
        self.0.remote(Self::ORIGIN, origin.to_str().unwrap()).unwrap();
        self
    }

    pub fn pointed(&self, url: &str) {
        self.0.remote_set_url(Self::ORIGIN, url).unwrap();
    }

    pub fn configured(&self, key: &str, value: &str) {
        self.0.config().unwrap().set_str(key, value).unwrap();
    }

    pub fn cloned_quietly(origin: &Path, dir: &Path, branch: &str) -> Fixture {
        let fixture = Self::working(dir, branch);
        fixture.0.config().unwrap().set_bool(Self::NO_REFLOG, false).unwrap();
        let fixture = fixture.linked(origin);
        fixture.0.find_remote(Self::ORIGIN).unwrap().fetch(&[] as &[&str], None, None).unwrap();
        let tracked = format!("{}/{branch}", Self::ORIGIN);
        fixture.tracked(branch, &tracked);
        fixture
    }

    fn tracked(&self, branch: &str, tracked: &str) {
        let tip = self.0.find_branch(tracked, git2::BranchType::Remote).unwrap().get().peel_to_commit().unwrap();
        self.0.branch(branch, &tip, false).unwrap().set_upstream(Some(tracked)).unwrap();
        self.0.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();
    }

    pub fn committed(&self, message: &str) -> String {
        self.committed_at(message, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().cast_signed())
    }

    pub fn committed_at(&self, message: &str, seconds: i64) -> String {
        let signature = git2::Signature::new(Self::NAME, Self::EMAIL, &git2::Time::new(seconds, 0)).unwrap();
        let parent = self.0.head().and_then(|head| head.peel_to_commit()).ok();
        let tree = parent
            .as_ref()
            .map_or_else(|| self.0.find_tree(self.0.treebuilder(None).unwrap().write().unwrap()), git2::Commit::tree);
        let made =
            self.0.commit(Some(Self::HEAD), &signature, &signature, message, &tree.unwrap(), &Vec::from_iter(&parent));
        let made = made.unwrap();
        let branch = self.0.head().unwrap().name().unwrap().to_owned();
        self.relogged(Self::HEAD, made, &signature);
        self.relogged(&branch, made, &signature);
        made.to_string()
    }

    fn relogged(&self, name: &str, made: git2::Oid, signature: &git2::Signature<'_>) {
        let mut log = self.0.reflog(name).unwrap();
        let newest = log.get(0).filter(|entry| entry.id_new() == made);
        if let Some(message) = newest.and_then(|entry| entry.message().unwrap().map(str::to_owned)) {
            log.remove(0, false).unwrap();
            log.append(made, signature, Some(&message)).unwrap();
            log.write().unwrap();
        }
    }

    pub fn pushed(&self, to: &str) {
        let from = self.0.head().unwrap().name().unwrap().to_owned();
        self.0.find_remote(Self::ORIGIN).unwrap().push(&[format!("+{from}:refs/heads/{to}")], None).unwrap();
    }

    pub fn head(&self) -> String {
        self.0.head().unwrap().target().unwrap().to_string()
    }
}
