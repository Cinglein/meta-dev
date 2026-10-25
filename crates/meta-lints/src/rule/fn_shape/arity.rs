use syn::{FnArg, Signature};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Arity(usize);

impl Arity {
    pub const fn new(parameters: usize) -> Arity {
        Arity(parameters)
    }

    pub fn counted_in(signature: &Signature) -> Arity {
        Arity(signature.inputs.iter().filter(|argument| matches!(argument, FnArg::Typed(_))).count())
    }
}
