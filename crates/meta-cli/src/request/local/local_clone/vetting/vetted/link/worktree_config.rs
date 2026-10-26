use std::str::FromStr;

use crate::request::local::local_clone::vetting::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeConfig(Vec<String>);

impl WorktreeConfig {
    const OPENING: char = '[';
    const REDIRECTING: [&'static str; 3] = ["[remote", "[url", "[include"];

    pub fn unredirected(&self) -> Result<(), Kept> {
        let redirecting = |section: &String| Self::REDIRECTING.iter().any(|name| section.starts_with(name));
        (!self.0.iter().any(redirecting)).then_some(()).ok_or(Kept::Redirected)
    }
}

impl FromStr for WorktreeConfig {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let sections = text.lines().map(str::trim).filter(|line| line.starts_with(Self::OPENING));
        Ok(Self(sections.map(str::to_ascii_lowercase).collect()))
    }
}
