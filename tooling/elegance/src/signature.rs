mod normalizer;

use std::fmt;

use crate::method_name::MethodName;
use normalizer::Normalizer;
use quote::ToTokens;
use syn::visit_mut::VisitMut;
use syn::{FnArg, Ident, ReturnType};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Signature {
    name: MethodName,
    text: String,
}

impl Signature {
    #[must_use]
    pub fn name(&self) -> &MethodName {
        &self.name
    }

    #[must_use]
    pub fn abstracted(signature: &syn::Signature, own: Option<&Ident>) -> Signature {
        let mut normalized = signature.clone();
        Normalizer::new(own).visit_signature_mut(&mut normalized);
        let inputs: Vec<String> = normalized.inputs.iter().map(Self::input).collect();
        let output = match &normalized.output {
            ReturnType::Default => String::new(),
            ReturnType::Type(_, ty) => format!(" -> {}", Self::spaced(ty)),
        };
        Signature {
            name: MethodName::new(&normalized.ident),
            text: format!("{}({}){output}", normalized.ident, inputs.join(", ")),
        }
    }

    fn input(argument: &FnArg) -> String {
        match argument {
            FnArg::Receiver(receiver) => Self::spaced(receiver),
            FnArg::Typed(typed) => Self::spaced(&typed.ty),
        }
    }

    fn spaced(tokens: &impl ToTokens) -> String {
        let rendered = tokens.to_token_stream().to_string();
        let previous = std::iter::once(None).chain(rendered.chars().map(Some));
        let next = rendered.chars().skip(1).map(Some).chain(std::iter::once(None));
        rendered
            .chars()
            .zip(previous)
            .zip(next)
            .filter(|((character, before), after)| *character != ' ' || Self::separates(*before, *after))
            .map(|((character, _), _)| character)
            .collect()
    }

    fn separates(before: Option<char>, after: Option<char>) -> bool {
        let word = |character: Option<char>| {
            character.is_some_and(|character| character.is_alphanumeric() || character == '_')
        };
        before == Some(',') || word(before) && word(after)
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}
