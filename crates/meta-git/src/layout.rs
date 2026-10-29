#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Bare,
    Working,
}

impl Layout {
    const INITIAL_HEAD: &'static str = "main";

    pub(crate) fn options(self) -> git2::RepositoryInitOptions {
        let mut options = git2::RepositoryInitOptions::new();
        options.bare(self == Layout::Bare).initial_head(Self::INITIAL_HEAD);
        options
    }
}
