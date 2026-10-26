#[derive(Deserialize)] pub struct Settings { schedule: Schedule, rounds: Count, plain: Plain } //~ settings-shape: struct is read
impl EmbeddedDefaults for Settings { const DEFAULTS: &str = ""; }
#[derive(Deserialize)] #[serde(default)] pub struct Schedule { batches: usize } //~ settings-shape: struct is read
#[derive(Deserialize)] pub struct Count(usize); #[derive(Debug)] pub struct Plain { x: usize }
#[derive(Deserialize)] pub struct Metadata { packages: Vec<Package> }
#[derive(Deserialize)] #[serde(deny_unknown_fields)] pub struct Strict { rounds: usize }
impl EmbeddedDefaults for Strict { const DEFAULTS: &str = ""; }
