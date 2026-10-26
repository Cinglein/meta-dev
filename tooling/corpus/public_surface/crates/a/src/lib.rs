mod x; use x::Private;
pub use x::{Used, Unused, Wired}; pub use x::Original as Seen; //~ public-surface: Unused is exported
