use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

use task::SpecSettings;

#[derive(Debug)]
pub struct Unconverted(String);

impl FromStr for Unconverted {
    type Err = Infallible;

    fn from_str(written: &str) -> Result<Unconverted, Infallible> {
        let marked = written.split_once(SpecSettings::UNCONVERTED).map(|(_, marked)| marked);
        let prose = marked.and_then(|marked| marked.split_once('\n')).map_or(written, |(_, prose)| prose);
        Ok(Unconverted(prose.to_owned()))
    }
}

impl fmt::Display for Unconverted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = self.0.lines().filter(|line| !line.is_empty()).count();
        writeln!(formatter, "{}{lines} lines", SpecSettings::UNCONVERTED)?;
        formatter.write_str(&self.0)
    }
}
