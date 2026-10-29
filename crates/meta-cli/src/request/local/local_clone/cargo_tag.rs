use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct CargoTag;

impl CargoTag {
    pub const FILE: &'static str = "CACHEDIR.TAG";
    const SIGNATURE: &'static str = "Signature: 8a477f597d28d172789f06886806bc55";
    const AUTHOR: &'static str = "created by cargo";

    pub fn marks(text: &str) -> bool {
        text.starts_with(Self::SIGNATURE) && text.contains(Self::AUTHOR)
    }

    pub fn tags(dir: &Path) -> bool {
        std::fs::read_to_string(dir.join(Self::FILE)).is_ok_and(|text| Self::marks(&text))
    }
}
