use std::path::Path;
use std::process::Command;

use meta_golden::Replay;
use meta_pull_query::LiveQuery;

pub struct RecordedGh(tempfile::TempDir);

impl RecordedGh {
    const REPLAY: Option<&str> = option_env!("CARGO_BIN_EXE_gh-replay");
    const UNBUILT: &str = "gh-replay is built with feature `replay`; run the tests with --all-features";
    const PROGRAM: &str = "gh-replay";
    const REPLIES: &str = "replies";
    const TRAIL: &str = "trail";
    const COPIES: [&str; 3] = ["a", "b", "c"];
    const GRAPHQL: [&str; 6] = ["api", "graphql", "-F", "owner={owner}", "-F", "name={repo}"];
    const QUERY: &str = "-f";
    const SUCCEEDED: &str = "0";

    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        [Self::REPLIES, Self::TRAIL].iter().try_for_each(|name| std::fs::create_dir(dir.path().join(name))).unwrap();
        Self(dir)
    }

    pub fn program() -> &'static str {
        Self::REPLAY.filter(|built| Path::new(built).exists()).expect(Self::UNBUILT)
    }

    pub fn graphql(&self, name: &str, linked: &[&str], out: &str) -> &Self {
        let query = LiveQuery::new(linked.iter().map(|number| number.parse().unwrap()).collect());
        let words = Self::GRAPHQL.map(String::from).into_iter().chain([Self::QUERY.into(), query.field()]);
        self.recorded(name, words, out)
    }

    pub fn answered(&self, name: &str, words: &[&str], out: &str) -> &Self {
        self.recorded(name, words.iter().map(|word| (*word).to_string()), out)
    }

    pub fn on<'command>(&self, command: &'command mut Command) -> &'command mut Command {
        command
            .env(Replay::RECORDINGS, self.0.path().join(Self::REPLIES))
            .env(Replay::TRAIL, self.0.path().join(Self::TRAIL))
    }

    fn recorded(&self, name: &str, words: impl Iterator<Item = String>, out: &str) -> &Self {
        let argv = [Self::PROGRAM.to_string()].into_iter().chain(words).collect::<Vec<_>>();
        let argv = serde_json::to_string(&argv).unwrap();
        let files = [("argv", argv.as_str()), ("out", out), ("err", ""), ("status", Self::SUCCEEDED)];
        let stems = Self::COPIES.map(|copy| self.0.path().join(Self::REPLIES).join(format!("{name}-{copy}")));
        let written =
            stems.iter().flat_map(|stem| files.map(|(extension, text)| (stem.with_extension(extension), text)));
        written.into_iter().try_for_each(|(file, text)| std::fs::write(file, text)).unwrap();
        self
    }
}
