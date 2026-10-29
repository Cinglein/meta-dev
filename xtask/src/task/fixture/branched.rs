use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branched {
    Tracked,
    Local,
}

impl Branched {
    pub fn applied(self, work: &Path) -> Option<()> {
        (self == Branched::Tracked).then(|| {
            let tip = fs::read_to_string(work.join(".git/refs/heads/change")).expect("read branch tip");
            let remote = work.join(".git/refs/remotes/origin");
            fs::create_dir_all(&remote).expect("remote refs dir");
            fs::write(remote.join("main"), tip).expect("write origin main");
            fs::write(remote.join("HEAD"), "ref: refs/remotes/origin/main\n").expect("write origin HEAD");
        })
    }
}
