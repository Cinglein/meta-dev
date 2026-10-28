use std::ffi::OsString;

use crate::request::local::local_clone::vetting::cause::Cause;
use crate::request::local::local_clone::vetting::kept::Kept;

pub struct FakeRoots<'host> {
    pseudo: &'host [&'static str],
    dangling: &'host [&'static str],
}

impl<'host> FakeRoots<'host> {
    const PAUSE_REFS: [&'static str; 4] = ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "REBASE_HEAD"];
    const HELD: &'static str = "c7\n";

    pub fn new(pseudo: &'host [&'static str], dangling: &'host [&'static str]) -> Self {
        Self { pseudo, dangling }
    }

    pub fn verified(&self, arguments: &[OsString]) -> Option<Result<String, Kept>> {
        let named: Vec<&OsString> =
            arguments.iter().filter(|argument| Self::PAUSE_REFS.iter().any(|name| *argument == *name)).collect();
        let first = named.first()?;
        let held = |named: &&&OsString| self.pseudo.iter().chain(self.dangling).any(|name| **named == *name);
        let leading = || named.iter().take_while(held).map(|_| Self::HELD).collect();
        let alone = || Some(first).filter(held).map(|_| Self::HELD.to_owned()).ok_or(Kept::Uninspected(Cause::new("")));
        Some(named.get(1).map_or_else(alone, |_| Ok(leading())))
    }
}
