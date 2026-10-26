mod concern;
mod issue;
mod task;

pub use concern::Concern;
pub use issue::Issue;
pub use task::Task;

pub trait Entity {
    const PREFIX: &'static str;
}
