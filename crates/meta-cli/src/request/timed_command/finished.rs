mod verdict;

use std::fmt;
use std::process::ExitStatus;

use crate::request::output::Listed;
use verdict::Verdict;

#[derive(Debug)]
pub struct Finished {
    status: ExitStatus,
    output: String,
    errors: String,
}

impl Finished {
    const SHOWN_REPLY_LEN: usize = 40;
    const NOT_FOUND: &'static str = "HTTP 404";

    pub fn new(status: ExitStatus, output: String, errors: String) -> Self {
        Self { status, output, errors }
    }

    pub fn succeeded(self) -> Result<String, Listed> {
        let complaint = self.complaint();
        self.status.success().then_some(self.output).ok_or(complaint)
    }

    pub fn not_found(&self) -> Option<()> {
        (!self.status.success() && self.errors.contains(Self::NOT_FOUND)).then_some(())
    }

    pub fn verdict(&self) -> Verdict {
        match (self.status.success(), self.errors.is_empty()) {
            (true, _) => Verdict::Clean,
            (false, true) => Verdict::Flagged,
            (false, false) => Verdict::Failed,
        }
    }

    pub fn output(&self) -> &str {
        &self.output
    }

    pub fn complaint(&self) -> Listed {
        Listed::new(self.errors.lines().next().unwrap_or_default())
    }
}

impl fmt::Display for Finished {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown: String = self.output.chars().take(Self::SHOWN_REPLY_LEN).collect();
        let reason = self
            .errors
            .lines()
            .next()
            .filter(|line| !line.is_empty())
            .map_or_else(|| format!("reply not JSON: {}", shown.trim()), str::to_owned);
        formatter.write_str(&reason)
    }
}
