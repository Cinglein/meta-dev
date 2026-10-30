use std::ffi::OsStr;
use std::path::Path;

use strum::{Display, EnumString, VariantArray};

use crate::span_kind::SpanKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, EnumString, VariantArray)]
#[strum(serialize_all = "lowercase")]
pub enum Program {
    Git,
    Gh,
    Meta,
    Lsof,
    Other,
}

impl Program {
    const GIT_DIRECTORY: &'static str = "-C";
    const DIRECTORY_LEN: usize = 2;
    const NETWORK_GIT: [&'static str; 5] = ["fetch", "push", "pull", "ls-remote", "clone"];

    #[must_use]
    pub fn named(path: &OsStr) -> Self {
        let name = Path::new(path).file_name().and_then(OsStr::to_str);
        name.and_then(|name| name.parse().ok()).unwrap_or(Self::Other)
    }

    #[must_use]
    pub fn spawned(self, args: &[&OsStr]) -> SpanKind {
        match self {
            Self::Gh => SpanKind::Network(self),
            Self::Git => Self::network_subcommand(args).map_or(SpanKind::Spawn(self), |_| SpanKind::Network(self)),
            Self::Meta | Self::Lsof | Self::Other => SpanKind::Spawn(self),
        }
    }

    fn network_subcommand<'args>(args: &[&'args OsStr]) -> Option<&'args str> {
        let directed = args.first().filter(|first| **first == Self::GIT_DIRECTORY);
        let word = args.get(directed.map_or(0, |_| Self::DIRECTORY_LEN)).and_then(|word| word.to_str());
        word.filter(|word| Self::NETWORK_GIT.contains(word))
    }
}
