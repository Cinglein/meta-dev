use std::path::Path;

#[derive(Debug)]
pub struct CargoTag;

impl CargoTag {
    const FILE: &'static str = "CACHEDIR.TAG";
    const SIGNATURE: &'static str = "Signature: 8a477f597d28d172789f06886806bc55";
    const AUTHOR: &'static str = "created by cargo";

    pub fn marks(dir: &Path) -> bool {
        std::fs::read_to_string(dir.join(Self::FILE))
            .is_ok_and(|tag| tag.starts_with(Self::SIGNATURE) && tag.contains(Self::AUTHOR))
    }
}
