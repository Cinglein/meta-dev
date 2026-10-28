use std::fs::Metadata;
use std::io::{self, ErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Missing,
    File,
    Dir,
    Admin,
    Link,
    Unreadable,
    Refused,
}

impl Kind {
    #[must_use]
    pub fn classified(meta: &Metadata) -> Kind {
        match meta.file_type() {
            kind if kind.is_symlink() => Kind::Link,
            kind if kind.is_dir() => Kind::Dir,
            _ => Kind::File,
        }
    }

    #[must_use]
    pub fn found(described: io::Result<Metadata>) -> Kind {
        match described {
            Ok(meta) => Kind::classified(&meta),
            Err(failure) if failure.kind() == ErrorKind::NotFound => Kind::Missing,
            Err(_) => Kind::Unreadable,
        }
    }
}
