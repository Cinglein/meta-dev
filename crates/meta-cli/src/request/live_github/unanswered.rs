use std::fmt;

use meta_run::Ran;

#[derive(Debug)]
pub struct Unanswered<'ran>(&'ran Ran);

impl<'ran> Unanswered<'ran> {
    const SHOWN_REPLY_LEN: usize = 40;
    const NOT_FOUND: &'static str = "HTTP 404";

    pub fn new(ran: &'ran Ran) -> Self {
        Self(ran)
    }

    pub fn not_found(&self) -> Option<()> {
        self.0.stderr().to_string().contains(Self::NOT_FOUND).then_some(())
    }
}

impl fmt::Display for Unanswered<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(complaint) = self.0.stderr().first_line() {
            formatter.write_str(complaint)
        } else {
            let shown: String = self.0.stdout().text().chars().take(Self::SHOWN_REPLY_LEN).collect();
            write!(formatter, "reply not JSON: {}", shown.trim())
        }
    }
}
