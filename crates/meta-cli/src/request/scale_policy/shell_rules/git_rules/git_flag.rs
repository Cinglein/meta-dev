use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct GitFlag(String);

impl GitFlag {
    const LONG: &'static str = "--";
    const SHORT: &'static str = "-";
    const VALUE: &'static str = "=";

    pub fn matches(&self, arg: &str) -> bool {
        match self.0.strip_prefix(Self::LONG) {
            Some(_) => {
                arg == self.0 || arg.strip_prefix(self.0.as_str()).is_some_and(|rest| rest.starts_with(Self::VALUE))
            }
            None => {
                arg.starts_with(Self::SHORT)
                    && !arg.starts_with(Self::LONG)
                    && arg.contains(self.0.trim_start_matches(Self::SHORT))
            }
        }
    }
}
