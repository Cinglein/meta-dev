pub struct Quiet;
// one //~ no-comments: comment
impl Quiet {
    /* two */ //~ no-comments: comment
    /// three //~ no-comments: comment
    pub fn speak(&self) {}
}
const URL: &str = "https://example.com";
const SLASH: char = '/';
