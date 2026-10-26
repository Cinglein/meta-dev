use std::ffi::OsString;
use std::fmt;
use std::path::{Component, Path, PathBuf};

use super::xml_text::XmlText;
use crate::request::local::LocalClone;
use crate::request::project::{HeartbeatSettings, Seconds};

#[derive(Debug)]
pub struct LaunchdPlist {
    program: PathBuf,
    checkout: PathBuf,
    log: PathBuf,
    interval: Seconds,
    search_path: OsString,
}

impl LaunchdPlist {
    const LABEL_PREFIX: &'static str = "meta.heartbeat";
    const LOG: &'static str = "meta-local/heartbeat.log";

    pub fn new(program: PathBuf, clone: &LocalClone, settings: &HeartbeatSettings, search_path: OsString) -> Self {
        Self {
            program,
            checkout: clone.checkout().to_path_buf(),
            log: clone.git_dir().join(Self::LOG),
            interval: settings.interval(),
            search_path,
        }
    }

    fn label(&self) -> String {
        self.checkout
            .components()
            .filter_map(|component| match component {
                Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
                _ => None,
            })
            .fold(Self::LABEL_PREFIX.to_owned(), |label, name| format!("{label}.{name}"))
    }

    fn text(path: &Path) -> String {
        XmlText::new(&path.to_string_lossy()).to_string()
    }
}

impl fmt::Display for LaunchdPlist {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = self.label();
        write!(
            formatter,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\">\n\
             <dict>\n\
             \t<key>Label</key>\n\
             \t<string>{label}</string>\n\
             \t<key>ProgramArguments</key>\n\
             \t<array>\n\
             \t\t<string>{program}</string>\n\
             \t\t<string>heartbeat</string>\n\
             \t\t<string>tick</string>\n\
             \t</array>\n\
             \t<key>WorkingDirectory</key>\n\
             \t<string>{checkout}</string>\n\
             \t<key>EnvironmentVariables</key>\n\
             \t<dict>\n\
             \t\t<key>PATH</key>\n\
             \t\t<string>{search_path}</string>\n\
             \t</dict>\n\
             \t<key>StartInterval</key>\n\
             \t<integer>{interval}</integer>\n\
             \t<key>RunAtLoad</key>\n\
             \t<true/>\n\
             \t<key>StandardOutPath</key>\n\
             \t<string>{log}</string>\n\
             \t<key>StandardErrorPath</key>\n\
             \t<string>{log}</string>\n\
             </dict>\n\
             </plist>\n",
            label = XmlText::new(&label),
            program = Self::text(&self.program),
            checkout = Self::text(&self.checkout),
            search_path = XmlText::new(&self.search_path.to_string_lossy()),
            interval = self.interval,
            log = Self::text(&self.log),
        )
    }
}
