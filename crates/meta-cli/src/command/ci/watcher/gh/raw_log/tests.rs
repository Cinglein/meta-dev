use super::RawLog;
use crate::command::project::LineCount;

const CLIPPY: &str = "ci\tRun cargo xtask ci\t2026-10-25T06:30:01.1000000Z    Compiling meta-log v0.0.0
ci\tRun cargo xtask ci\t2026-10-25T06:30:02.1000000Z     Checking meta-cli v0.0.0
ci\tRun cargo xtask ci\t2026-10-25T06:30:05.1000000Z \u{1b}[1m\u{1b}[31merror\u{1b}[0m: this method chain can be written more clearly with `if .. else ..`
ci\tRun cargo xtask ci\t2026-10-25T06:30:05.1000000Z   --> crates/meta-log/src/git_log/remote.rs:43:9
ci\tRun cargo xtask ci\t2026-10-25T06:30:05.2000000Z error: could not compile `meta-log` (lib) due to 1 previous error
ci\tRun cargo xtask ci\t2026-10-25T06:30:05.3000000Z ci: clippy 2.3s
ci\tRun cargo xtask ci\t2026-10-25T06:30:05.4000000Z ci: clippy failed
ci\tRun cargo xtask ci\t2026-10-25T06:30:05.5000000Z ##[error]Process completed with exit code 1.";
const WIDE: &str = "limit = 50\ncontext = 1\n";
const NARROW: &str = "limit = 3\ncontext = 0\n";
const HEADED_ERROR: &str = "== ci / Run cargo xtask ci ==\nerror: this method chain";
const LOCATION: &str = "remote.rs:43:9";
const NOISE: &[&str] = &["Compiling", "2026-10-25T", "\u{1b}"];
const CAPPED: &str = "[kept 3 of 8 log lines; cut setup, passing output and anything beyond 3 lines]";

#[derive(serde::Deserialize)]
struct Limits {
    limit: LineCount,
    context: LineCount,
}

impl Limits {
    fn excerpt(text: &str) -> String {
        let limits: Self = toml::from_str(text).unwrap();
        RawLog::new(CLIPPY.to_owned()).excerpt(limits.limit, limits.context).to_string()
    }
}

#[test]
fn keeps_error_blocks_without_setup_noise_timestamps_or_colour() {
    let excerpt = Limits::excerpt(WIDE);
    assert!(excerpt.contains(HEADED_ERROR) && excerpt.contains(LOCATION));
    assert!(NOISE.iter().all(|noise| !excerpt.contains(noise)));
}

#[test]
fn caps_the_excerpt_and_says_how_much_was_kept() {
    assert!(Limits::excerpt(NARROW).ends_with(CAPPED));
}
