use std::ffi::OsStr;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefLine {
    object: String,
    name: String,
}

impl RefLine {
    const HEAD: &'static str = "HEAD";

    pub fn head(object: &str) -> Self {
        Self { object: object.trim().to_owned(), name: Self::HEAD.to_owned() }
    }

    pub fn object(&self) -> &OsStr {
        OsStr::new(&self.object)
    }

    pub fn deletion(&self) -> impl Iterator<Item = &OsStr> {
        [&self.name, &self.object].into_iter().map(OsStr::new)
    }

    pub fn named(&self, name: &str) -> Option<&Self> {
        (self.name == name).then_some(self)
    }

    pub fn diverges_from(&self, remote: &Self) -> bool {
        self.name != Self::HEAD && self.name == remote.name && self.object != remote.object
    }
}

impl FromStr for RefLine {
    type Err = ();

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        match line.split_whitespace().collect::<Vec<_>>().as_slice() {
            [object, name] => Ok(Self { object: (*object).to_owned(), name: (*name).to_owned() }),
            _ => Err(()),
        }
    }
}
