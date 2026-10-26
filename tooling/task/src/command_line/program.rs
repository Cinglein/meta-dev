use std::env;
use std::fmt;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Program {
    Cargo,
    Git,
    GitHub,
    Executable(PathBuf),
}

impl fmt::Display for Program {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Program::Cargo => {
                formatter.write_str(&env::var("CARGO").unwrap_or_else(|_| String::from("cargo")))
            }
            Program::Git => formatter.write_str("git"),
            Program::GitHub => formatter.write_str("gh"),
            Program::Executable(path) => write!(formatter, "{}", path.display()),
        }
    }
}
