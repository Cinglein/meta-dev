mod ident;

pub use ident::Ident;

pub type Staged<Verdict, Ask> = std::ops::ControlFlow<Verdict, Ask>;
