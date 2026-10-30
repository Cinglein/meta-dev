use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct Flag(String);

impl Flag {
    const LONG: &'static str = "--";
    const SHORT: &'static str = "-";
    const VALUE: &'static str = "=";
    const TERMINATORS: &'static [&'static str] = &[";", "+"];

    pub fn names(&self, arg: &str) -> bool {
        let body = self.0.trim_start_matches(Self::SHORT);
        match (self.0.starts_with(Self::LONG), body.len()) {
            (false, 1) => arg.starts_with(Self::SHORT) && !arg.starts_with(Self::LONG) && arg.contains(body),
            _ => arg == self.0,
        }
    }

    pub fn joined<'arg>(&self, arg: &'arg str) -> Option<&'arg str> {
        arg.strip_prefix(self.0.as_str()).and_then(|rest| rest.strip_prefix(Self::VALUE))
    }

    pub fn command_after(flags: &[Self], args: &[String]) -> Option<String> {
        let joined = args.iter().find_map(|arg| flags.iter().find_map(|flag| flag.joined(arg)));
        let words = args.split(|arg| flags.iter().any(|flag| flag.names(arg))).nth(1);
        let separate = words.map(|words| {
            words.iter().take_while(|word| !Self::TERMINATORS.contains(&word.as_str())).cloned().collect::<Vec<_>>()
        });
        joined.map(str::to_owned).or_else(|| separate.map(|words| words.join(" ")))
    }
}
