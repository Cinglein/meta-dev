use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub struct Unconverted(String);

impl Unconverted {
    const MARKER: &str = "# Not yet converted: hand-written prose, ";
}

impl FromStr for Unconverted {
    type Err = Infallible;

    fn from_str(written: &str) -> Result<Unconverted, Infallible> {
        let marked = written.find(Self::MARKER).and_then(|start| written.get(start..));
        let prose = marked.and_then(|marked| marked.split_once('\n')).map_or(written, |(_, prose)| prose);
        Ok(Unconverted(prose.to_owned()))
    }
}

impl fmt::Display for Unconverted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = self.0.lines().filter(|line| !line.is_empty()).count();
        writeln!(formatter, "{}{lines} lines", Self::MARKER)?;
        formatter.write_str(&self.0)
    }
}
