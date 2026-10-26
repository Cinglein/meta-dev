mod request; //~ thin-main: main.rs holds only fn main and imports
use std::process::ExitCode;
struct Flags; //~ thin-main: main.rs holds only fn main and imports
fn main() {}
fn parse_flags() {} //~ thin-main: main.rs holds only fn main and imports //~ no-free-fns: fn parse_flags
