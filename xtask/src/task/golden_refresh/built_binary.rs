use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BuiltBinary {
    executable: Option<PathBuf>,
}

impl BuiltBinary {
    pub fn located(messages: &str, name: &str) -> Option<PathBuf> {
        let built = messages.lines().filter_map(|line| serde_json::from_str::<Self>(line).ok());
        built.filter_map(|built| built.executable).find(|path| path.file_name().is_some_and(|file| file == name))
    }

    pub fn dir(executable: &Path) -> &Path {
        executable.parent().unwrap_or(executable)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::BuiltBinary;

    const MESSAGES: &str =
        "{\"executable\":null}\nnot json\n{\"executable\":\"/t/debug/other\"}\n{\"executable\":\"/t/debug/meta\"}\n";
    const FOUND: [&str; 3] = ["meta", "/t/debug/meta", "/t/debug"];

    #[test]
    fn the_named_executable_is_found_among_cargo_messages_and_its_dir_holds_it() {
        let [name, executable, dir] = FOUND;
        let found = BuiltBinary::located(MESSAGES, name);
        assert!(
            found.as_deref() == Some(Path::new(executable))
                && found.as_deref().map(BuiltBinary::dir) == Some(Path::new(dir))
        );
    }
}
