mod concern;
mod issue;
mod project;
mod task;

pub use concern::Concern;
pub use issue::Issue;
pub use project::Project;
pub use task::Task;

pub trait Entity {
    const PREFIX: &'static str;
}
