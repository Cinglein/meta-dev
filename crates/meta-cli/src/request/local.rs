mod local_clone;
mod local_file;
mod local_settings;
mod local_trail;
mod recorded_worktree;
mod trail_entry;
mod worktrees;

pub use local_clone::LocalClone;

#[cfg(test)]
mod tests;
