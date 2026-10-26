#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    Missing,
    Unreadable,
    Symlink,
    Dir,
    File,
}
