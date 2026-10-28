use std::fs::OpenOptions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opening {
    Read,
    Truncate,
    Append,
    Held,
}

impl Opening {
    pub(crate) fn options(self) -> OpenOptions {
        let mut options = OpenOptions::new();
        match self {
            Opening::Read => options.read(true),
            Opening::Truncate => options.write(true).create(true).truncate(true),
            Opening::Append => options.append(true).create(true),
            Opening::Held => options.read(true).write(true).create(true).truncate(false),
        }
        .clone()
    }
}
