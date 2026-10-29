use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct OpenAlerts(Vec<String>);

impl OpenAlerts {
    pub const NAME: &str = "dependabot alerts";
}

impl FromStr for OpenAlerts {
    type Err = Infallible;

    fn from_str(listed: &str) -> Result<OpenAlerts, Infallible> {
        Ok(OpenAlerts(listed.lines().filter(|line| !line.trim().is_empty()).map(str::to_owned).collect()))
    }
}

impl fmt::Display for OpenAlerts {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: on, {} open", Self::NAME, self.0.len())?;
        self.0.iter().try_for_each(|alert| write!(formatter, "\n{}: {alert}", Self::NAME))
    }
}

#[cfg(test)]
mod tests {
    use meta_golden::Golden;

    use super::OpenAlerts;

    const LISTED: &str = "high cargo time GHSA-aaaa-bbbb-cccc https://example.invalid/1\n\nlow actions checkout GHSA-dddd-eeee-ffff https://example.invalid/2\n";

    #[test]
    fn open_alerts_are_counted_and_each_shown_on_its_own_line() {
        let Ok(open) = LISTED.parse::<OpenAlerts>();
        Golden::for_test(module_path!(), "open").checked(&format!("{open}\n{}\n", OpenAlerts(Vec::new()))).unwrap();
    }
}
