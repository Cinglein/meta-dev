use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct TrackedPath<'path>(&'path Path);

impl<'path> TrackedPath<'path> {
    #[must_use]
    pub fn new(path: &'path Path) -> TrackedPath<'path> {
        TrackedPath(path)
    }

    #[must_use]
    pub fn path(&self) -> &'path Path {
        self.0
    }
}
