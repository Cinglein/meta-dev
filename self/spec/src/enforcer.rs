mod test_names;

use std::fmt;
use std::fs;
use std::path::Path;

use test_names::TestNames;

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
        let names = fs::read_to_string(root.join(self.file)).map(|text| text.parse::<TestNames>());
        names.is_ok_and(|names| names.is_ok_and(|names| names.holds(self.name)))
    }
}

impl fmt::Display for Enforcer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.file, self.name)
    }
}
