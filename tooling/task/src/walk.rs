use proc_macro2::{TokenStream, TokenTree};
use syn::{Attribute, ItemImpl};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Walk {
    Entered,
    Skipped,
}

impl Walk {
    #[must_use]
    pub fn inherent_impls(item: &ItemImpl) -> Walk {
        match item.trait_ {
            None => Walk::Entered,
            Some(_) => Walk::Skipped,
        }
    }

    #[must_use]
    pub fn outside_tests(attributes: &[Attribute]) -> Walk {
        if attributes.iter().any(Self::is_cfg_mentioning_test) {
            Walk::Skipped
        } else {
            Walk::Entered
        }
    }

    fn is_cfg_mentioning_test(attribute: &Attribute) -> bool {
        attribute.path().is_ident("cfg")
            && attribute
                .meta
                .require_list()
                .is_ok_and(|list| Self::mentions_test(list.tokens.clone()))
    }

    fn mentions_test(tokens: TokenStream) -> bool {
        tokens.into_iter().any(|token| match token {
            TokenTree::Ident(ident) => ident == "test",
            TokenTree::Group(group) => Self::mentions_test(group.stream()),
            TokenTree::Punct(_) | TokenTree::Literal(_) => false,
        })
    }
}
