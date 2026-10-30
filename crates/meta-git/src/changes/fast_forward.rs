use std::ffi::OsString;
use std::fmt;
use std::io::Write as _;

use crate::object_id::ObjectId;
use crate::ref_name::RefName;
use crate::remote_change::RemoteChange;
use crate::remote_name::RemoteName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FastForward {
    id: ObjectId,
    to: RefName,
}

impl FastForward {
    const REJECTED: &'static str = "rejected: the remote ref moved and is not an ancestor (non-fast-forward)";
    const MOVED: &'static str = "meta: fast-forward";
    const HOOKS: &'static str = "hooks";
    const SAMPLE: &'static str = "sample";
    const RECEIVE: &'static str = "^receive\\.";
    const HOOKS_PATH: &'static str = "core.hooksPath";
    const HEAD: &'static str = "HEAD";

    #[must_use]
    pub fn new(id: ObjectId, to: RefName) -> FastForward {
        FastForward { id, to }
    }

    fn packed(&self, here: &git2::Repository, there: &git2::Repository) -> Result<(), git2::Error> {
        let mut walk = here.revwalk()?;
        walk.push(self.id.oid())?;
        let known = there.references()?.filter_map(|reference| reference.ok()?.target());
        known.filter(|tip| here.find_commit(*tip).is_ok()).try_for_each(|tip| walk.hide(tip))?;
        let mut builder = here.packbuilder()?;
        builder.insert_walk(&mut walk)?;
        match builder.object_count() {
            0 => Ok(()),
            _ => Self::written(&mut builder, there),
        }
    }

    fn written(builder: &mut git2::PackBuilder<'_>, there: &git2::Repository) -> Result<(), git2::Error> {
        let odb = there.odb()?;
        let mut writer = odb.packwriter()?;
        builder.foreach(|chunk| writer.write_all(chunk).is_ok())?;
        writer.commit().map(drop)
    }

    fn current(&self, there: &git2::Repository) -> Result<Option<git2::Oid>, git2::Error> {
        match there.refname_to_id(self.to.as_str()) {
            Ok(current) => Ok(Some(current)),
            Err(error) if error.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }
}

impl RemoteChange for FastForward {
    const HELPER: &'static str = "receivepack";

    fn words(&self, remote: &RemoteName) -> Vec<OsString> {
        ["push", "--quiet"]
            .map(OsString::from)
            .into_iter()
            .chain([OsString::from(remote.as_ref()), OsString::from(self.to_string())])
            .collect()
    }

    fn in_process(&self, there: git2::Repository) -> Option<git2::Repository> {
        self.unguarded(&there).map(|()| there)
    }

    fn moved(&self, here: &git2::Repository, there: &git2::Repository) -> Result<(), git2::Error> {
        self.packed(here, there)?;
        let id = self.id.oid();
        let name = self.to.as_str();
        match self.current(there)? {
            Some(current) if current != id && !there.graph_descendant_of(id, current)? => {
                Err(git2::Error::from_str(Self::REJECTED))
            }
            Some(current) => there.reference_matching(name, id, true, current, Self::MOVED).map(drop),
            None => there.reference(name, id, false, Self::MOVED).map(drop),
        }
    }
}

impl FastForward {
    fn unguarded(&self, there: &git2::Repository) -> Option<()> {
        let hooks = std::fs::read_dir(there.path().join(Self::HOOKS)).into_iter().flatten().flatten();
        let hooked =
            hooks.map(|hook| hook.path()).find(|hook| hook.extension().is_none_or(|kind| kind != Self::SAMPLE));
        let config = there.config().ok()?;
        let received = config.entries(Some(Self::RECEIVE)).ok()?.next().map(drop);
        let hooks_path = config.get_path(Self::HOOKS_PATH).ok();
        let head = there.find_reference(Self::HEAD).ok()?;
        let target = self.to.as_str().as_bytes();
        let checked_out = head.symbolic_target_bytes().filter(|named| !there.is_bare() && *named == target);
        (hooked.is_none() && received.is_none() && hooks_path.is_none() && checked_out.is_none()).then_some(())
    }
}

impl fmt::Display for FastForward {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.id, self.to)
    }
}
