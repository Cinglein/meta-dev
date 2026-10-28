pub struct Door; impl Door { pub fn opened(path: &std::path::Path) -> std::io::Result<std::fs::File> { std::fs::File::open(path) } }
