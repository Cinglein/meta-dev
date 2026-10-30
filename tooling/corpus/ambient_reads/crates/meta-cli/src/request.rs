pub struct Request;
impl Request {
    pub fn actor() -> Option<String> {
        std::env::var(ACTOR).ok() //~ ambient-reads: reads the process environment outside main
    }
    pub fn here() -> Option<std::path::PathBuf> {
        std::env::current_dir().ok() //~ ambient-reads: reads the process environment outside main
    }
}
