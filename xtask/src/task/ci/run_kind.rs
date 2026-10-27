use std::collections::BTreeMap;
use std::ffi::OsString;

use crate::task::ci::head_branch::HeadBranch;
use crate::task::ci::untracked::Untracked;
use crate::task::failure::Failure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunKind {
    Pull(Option<HeadBranch>),
    Other,
}

impl RunKind {
    pub const GATE_BRANCH: &str = "META_GATE_BRANCH";
    const EVENT: &str = "GITHUB_EVENT_NAME";
    const PULL_EVENT: &str = "pull_request";
    const HEAD: &str = "GITHUB_HEAD_REF";
    const NUMBER: &str = "PULL_REQUEST";

    pub fn from_ambient(ambient: &BTreeMap<String, OsString>) -> RunKind {
        let set = |name: &str| ambient.get(name).map(|value| value.to_string_lossy().into_owned());
        let given = |name: &str| set(name).filter(|value| !value.is_empty());
        let head = given(Self::HEAD).or_else(|| given(Self::GATE_BRANCH));
        let pulled = given(Self::EVENT).filter(|event| event.starts_with(Self::PULL_EVENT));
        let signal = pulled.or(given(Self::NUMBER)).or(set(Self::GATE_BRANCH)).or(head.clone());
        let Ok(branch) = head.map(|head| head.parse()).transpose();
        signal.map_or(RunKind::Other, |_| RunKind::Pull(branch))
    }

    pub fn by_kind<T, R>(&self, target: &mut T, pull: impl FnOnce(&mut T) -> R, other: impl FnOnce(&mut T) -> R) -> R {
        match self {
            RunKind::Pull(_) => pull(target),
            RunKind::Other => other(target),
        }
    }

    pub fn refuse(&self, judge: impl FnOnce(Option<&HeadBranch>) -> Result<(), Untracked>) -> Result<(), Failure> {
        match self {
            RunKind::Pull(head) => judge(head.as_ref()).map_err(Failure::Untracked),
            RunKind::Other => Ok(()),
        }
    }
}
