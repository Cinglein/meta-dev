use strum::Display;

use task::UsePath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Effect {
    Process,
    Files,
    Environment,
    Clock,
    Network,
    Parsing,
}

impl Effect {
    const SEPARATOR: &str = "::";
    const FUNCTIONS: [(&str, Effect); 56] = [
        ("std::process::Command::new", Effect::Process),
        ("std::process::exit", Effect::Process),
        ("std::process::abort", Effect::Process),
        ("std::process::id", Effect::Process),
        ("std::fs::canonicalize", Effect::Files),
        ("std::fs::copy", Effect::Files),
        ("std::fs::create_dir", Effect::Files),
        ("std::fs::create_dir_all", Effect::Files),
        ("std::fs::exists", Effect::Files),
        ("std::fs::hard_link", Effect::Files),
        ("std::fs::metadata", Effect::Files),
        ("std::fs::read", Effect::Files),
        ("std::fs::read_dir", Effect::Files),
        ("std::fs::read_link", Effect::Files),
        ("std::fs::read_to_string", Effect::Files),
        ("std::fs::remove_dir", Effect::Files),
        ("std::fs::remove_dir_all", Effect::Files),
        ("std::fs::remove_file", Effect::Files),
        ("std::fs::rename", Effect::Files),
        ("std::fs::set_permissions", Effect::Files),
        ("std::fs::symlink_metadata", Effect::Files),
        ("std::fs::write", Effect::Files),
        ("std::fs::File::open", Effect::Files),
        ("std::fs::File::create", Effect::Files),
        ("std::fs::File::create_new", Effect::Files),
        ("std::fs::OpenOptions::new", Effect::Files),
        ("std::env::args", Effect::Environment),
        ("std::env::args_os", Effect::Environment),
        ("std::env::current_dir", Effect::Environment),
        ("std::env::current_exe", Effect::Environment),
        ("std::env::home_dir", Effect::Environment),
        ("std::env::remove_var", Effect::Environment),
        ("std::env::set_current_dir", Effect::Environment),
        ("std::env::set_var", Effect::Environment),
        ("std::env::temp_dir", Effect::Environment),
        ("std::env::var", Effect::Environment),
        ("std::env::var_os", Effect::Environment),
        ("std::env::vars", Effect::Environment),
        ("std::env::vars_os", Effect::Environment),
        ("std::time::SystemTime::now", Effect::Clock),
        ("std::time::Instant::now", Effect::Clock),
        ("std::thread::sleep", Effect::Clock),
        ("chrono::Utc::now", Effect::Clock),
        ("chrono::Local::now", Effect::Clock),
        ("std::net::TcpStream::connect", Effect::Network),
        ("std::net::TcpStream::connect_timeout", Effect::Network),
        ("std::net::TcpListener::bind", Effect::Network),
        ("std::net::UdpSocket::bind", Effect::Network),
        ("toml::from_str", Effect::Parsing),
        ("toml::Table::from_str", Effect::Parsing),
        ("toml::Value::from_str", Effect::Parsing),
        ("serde_json::from_str", Effect::Parsing),
        ("serde_json::from_slice", Effect::Parsing),
        ("serde_json::from_reader", Effect::Parsing),
        ("serde_json::from_value", Effect::Parsing),
        ("serde_json::Value::from_str", Effect::Parsing),
    ];

    #[must_use]
    pub fn performed_by(function: &UsePath) -> Option<Effect> {
        Self::FUNCTIONS
            .iter()
            .find(|(listed, _)| {
                function.segments().len() == listed.split(Self::SEPARATOR).count()
                    && function
                        .segments()
                        .iter()
                        .zip(listed.split(Self::SEPARATOR))
                        .all(|(segment, spelled)| *segment == spelled)
            })
            .map(|(_, effect)| *effect)
    }
}
