#![allow(clippy::unwrap_used, clippy::expect_used)]

mod ci;
#[allow(dead_code)]
#[path = "../it/fixture.rs"]
mod fixture;
mod hooks;
#[path = "../it/meta_command.rs"]
mod meta_command;
