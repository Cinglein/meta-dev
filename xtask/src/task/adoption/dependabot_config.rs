use std::convert::Infallible;
use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;

use crate::task::adoption::working_directory::WorkingDirectory;
use crate::task::adoption::yaml_node::YamlNode;

#[derive(Debug, Default)]
pub struct DependabotConfig(YamlNode);

impl DependabotConfig {
    pub const FILE: &str = ".github/dependabot.yml";
    const UPDATES: &str = "updates";
    const ECOSYSTEM: &str = "package-ecosystem";
    const CARGO: &str = "cargo";
    const DIRECTORY: &str = "directory";
    const DIRECTORIES: &str = "directories";
    const QUOTES: [char; 2] = ['"', '\''];
    const ROOT: char = '/';

    pub fn read(path: &Path) -> io::Result<DependabotConfig> {
        let text = match fs::read_to_string(path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
            read => read,
        }?;
        let Ok(config) = text.parse();
        Ok(config)
    }

    pub fn covers(&self, directory: &WorkingDirectory) -> bool {
        let cargo = |update: &&YamlNode| update.scalar_of(Self::ECOSYSTEM).trim_matches(Self::QUOTES) == Self::CARGO;
        self.0
            .items(Self::UPDATES)
            .iter()
            .filter(cargo)
            .flat_map(|update| {
                let single = Some(update.scalar_of(Self::DIRECTORY)).filter(|single| !single.is_empty());
                update.items(Self::DIRECTORIES).iter().map(YamlNode::name).chain(single)
            })
            .filter_map(|listed| listed.trim_matches(Self::QUOTES).trim_start_matches(Self::ROOT).parse().ok())
            .any(|listed: WorkingDirectory| listed == *directory)
    }
}

impl FromStr for DependabotConfig {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<DependabotConfig, Infallible> {
        let Ok(root) = text.parse();
        Ok(DependabotConfig(root))
    }
}
