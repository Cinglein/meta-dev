#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Bare,
    Working,
}

impl Layout {
    pub(crate) fn core(self) -> &'static str {
        match self {
            Layout::Bare => {
                "[core]\n\trepositoryformatversion = 1\n\tbare = true\n[extensions]\n\trefStorage = files\n"
            }
            Layout::Working => {
                "[core]\n\trepositoryformatversion = 1\n\tbare = false\n[extensions]\n\trefStorage = files\n"
            }
        }
    }
}
