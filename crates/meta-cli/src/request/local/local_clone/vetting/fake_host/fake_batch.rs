use std::fmt;

pub struct FakeBatch<'host> {
    pseudo: &'host [&'static str],
    head: String,
    input: &'host str,
}

impl<'host> FakeBatch<'host> {
    const PEELED: &'static str = "^{commit}";
    const ROOT_HEAD: &'static str = "HEAD";
    const HELD: &'static str = "c7";

    pub fn new(pseudo: &'host [&'static str], head: String, input: &'host str) -> Self {
        Self { pseudo, head, input }
    }

    fn found(&self, name: &str) -> Option<&str> {
        let held = self.pseudo.iter().find(|pseudo| **pseudo == name).map(|_| Self::HELD);
        (name == Self::ROOT_HEAD).then(|| self.head.lines().next().unwrap_or_default()).or(held)
    }
}

impl fmt::Display for FakeBatch<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.input.lines().try_for_each(|line| {
            let name = line.strip_suffix(Self::PEELED).unwrap_or(line);
            match self.found(name) {
                Some(found) => writeln!(formatter, "{found}"),
                None => writeln!(formatter, "{line} missing"),
            }
        })
    }
}
