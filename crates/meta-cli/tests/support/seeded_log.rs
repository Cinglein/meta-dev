use std::fmt;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

pub const TASK: &str = "task-00000000000000a1";

pub struct SeededLog(Vec<&'static str>);

impl SeededLog {
    const EVENTS: [&'static str; 3] = [
        r#"{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"SchemaVersion","version":2}"#,
        r#"{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000a1","text":"x","division":"features-and-maintenance"}"#,
        r#"{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskClaimed","task":"task-00000000000000a1"}"#,
    ];

    pub fn through(kind: &str) -> Self {
        let tag = format!("\"kind\":\"{kind}\"");
        let last = Self::EVENTS.iter().position(|event| event.contains(&tag)).map_or(Self::EVENTS.len(), |at| at + 1);
        Self(Self::EVENTS.iter().take(last).copied().collect())
    }

    pub fn plant(&self, bare: &Path) {
        let mut command = Command::new("git");
        let mut child =
            command.arg("-C").arg(bare).args(["fast-import", "--quiet"]).stdin(Stdio::piped()).spawn().unwrap();
        child.stdin.take().unwrap().write_all(self.to_string().as_bytes()).unwrap();
        assert!(child.wait().unwrap().success());
    }
}

impl fmt::Display for SeededLog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let events = [self.0.join("\n"), String::new()].join("\n");
        write!(
            formatter,
            "commit refs/heads/meta/log\ncommitter t <t@meta.invalid> 1700000000 +0000\ndata 4\nseed\nM 100644 inline events.jsonl\ndata {}\n{events}\n",
            events.len()
        )
    }
}
