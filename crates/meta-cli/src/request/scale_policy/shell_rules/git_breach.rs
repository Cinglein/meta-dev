use std::fmt;

#[derive(Debug)]
pub struct GitBreach {
    subcommand: String,
    flag: Option<String>,
}

impl GitBreach {
    pub fn new(subcommand: &str, flag: Option<&str>) -> Self {
        Self { subcommand: subcommand.to_owned(), flag: flag.map(str::to_owned) }
    }
}

impl fmt::Display for GitBreach {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "git {}", self.subcommand)?;
        self.flag.iter().try_for_each(|flag| write!(formatter, " {flag}"))
    }
}
