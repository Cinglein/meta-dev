use meta_door::kinds::Kind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    Missing,
    Unreadable,
    Symlink,
    Dir,
    File,
}

impl From<Kind> for EntryKind {
    fn from(kind: Kind) -> EntryKind {
        match kind {
            Kind::Missing => EntryKind::Missing,
            Kind::File => EntryKind::File,
            Kind::Dir => EntryKind::Dir,
            Kind::Link => EntryKind::Symlink,
            Kind::Unreadable | Kind::Refused => EntryKind::Unreadable,
        }
    }
}
