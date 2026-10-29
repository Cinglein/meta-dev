#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Invocation(String);

impl Invocation {
    const META: &'static str = "meta-";
    const RUN: char = '.';

    pub(crate) fn tagged(parent: &str) -> Option<Invocation> {
        parent.starts_with(Self::META).then(|| Invocation(parent.to_owned()))
    }

    pub(crate) fn command(&self) -> &str {
        let tag = self.0.strip_prefix(Self::META).unwrap_or(&self.0);
        tag.split(Self::RUN).next().unwrap_or(tag)
    }
}
