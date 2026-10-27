use std::ffi::OsString;

use crate::request::local::local_clone::vetting::cause::Cause;
use crate::request::local::local_clone::vetting::kept::Kept;

pub struct FakeRoots<'host> {
    pseudo: &'host [&'static str],
    dangling: &'host [&'static str],
}

impl<'host> FakeRoots<'host> {
    const PEELED: &'static str = "^{commit}";
    const ROOT_HEAD: &'static str = "HEAD";
    const HELD: &'static str = "c7\n";

    pub fn new(pseudo: &'host [&'static str], dangling: &'host [&'static str]) -> Self {
        Self { pseudo, dangling }
    }

    pub fn verified(&self, arguments: &[OsString]) -> Option<Result<String, Kept>> {
        let root = |argument: &OsString| {
            let name = argument.to_string_lossy();
            let name = name.strip_suffix(Self::PEELED).unwrap_or(&name).to_owned();
            (name != Self::ROOT_HEAD
                && !name.is_empty()
                && name.chars().all(|letter| letter.is_ascii_uppercase() || letter == '_' || letter == '-'))
            .then_some(name)
        };
        let named = arguments.iter().find_map(root)?;
        Some(
            self.pseudo
                .iter()
                .chain(self.dangling)
                .any(|pseudo| *pseudo == named)
                .then(|| Self::HELD.to_owned())
                .ok_or_else(|| Kept::Uninspected(Cause::new(""))),
        )
    }
}
