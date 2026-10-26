use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TestFn {
    file: &'static str,
    name: &'static str,
}

impl TestFn {
    #[must_use]
    pub const fn new(file: &'static str, name: &'static str) -> TestFn {
        TestFn { file, name }
    }

    #[must_use]
    pub fn exists_under(&self, root: &Path) -> bool {
        fs::read_to_string(root.join(self.file)).is_ok_and(|text| text.contains(&format!("fn {}(", self.name)))
    }
}

impl fmt::Display for TestFn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} in {}", self.name, self.file)
    }
}
