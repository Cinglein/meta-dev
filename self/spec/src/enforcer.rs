use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct Enforcer {
    file: &'static str,
    name: &'static str,
}

impl Enforcer {
    #[must_use]
    pub const fn new(file: &'static str, name: &'static str) -> Enforcer {
        Enforcer { file, name }
    }

    #[must_use]
    pub fn found(&self, root: &Path) -> bool {
        fs::read_to_string(root.join(self.file)).is_ok_and(|text| text.contains(self.name))
    }
}

impl fmt::Display for Enforcer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.file, self.name)
    }
}
