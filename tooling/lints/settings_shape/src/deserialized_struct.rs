use std::path::PathBuf;

use proc_macro2::TokenTree;
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{Attribute, Fields, ItemStruct, Path, Token, TypePath};

use super::keys::Keys;
use task::LineNumber;
use task::Name;

#[derive(Debug)]
pub struct DeserializedStruct {
    path: PathBuf,
    line: LineNumber,
    name: Name,
    field_types: Vec<Name>,
    keys: Keys,
}

impl DeserializedStruct {
    const KEY_SHAPES: [&str; 4] = ["deny_unknown_fields", "transparent", "try_from", "from"];

    pub fn from_item(path: &std::path::Path, item: &ItemStruct) -> Option<DeserializedStruct> {
        (matches!(item.fields, Fields::Named(_)) && Self::derives_deserialize(&item.attrs)).then(
            || {
                let mut types = FieldTypes::default();
                types.visit_fields(&item.fields);
                DeserializedStruct {
                    path: path.to_owned(),
                    line: LineNumber::from(item.ident.span()),
                    name: Name::from(&item.ident),
                    field_types: types.0,
                    keys: if item.attrs.iter().any(Self::shapes_its_keys) {
                        Keys::Denied
                    } else {
                        Keys::Open
                    },
                }
            },
        )
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn line(&self) -> LineNumber {
        self.line
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn field_types(&self) -> &[Name] {
        &self.field_types
    }

    pub fn keys(&self) -> Keys {
        self.keys
    }

    fn derives_deserialize(attributes: &[Attribute]) -> bool {
        attributes.iter().any(|attribute| {
            attribute.path().is_ident("derive")
                && attribute
                    .parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
                    .is_ok_and(|paths| paths.iter().any(Self::names_deserialize))
        })
    }

    fn names_deserialize(path: &Path) -> bool {
        path.segments
            .last()
            .is_some_and(|segment| segment.ident == "Deserialize")
    }

    fn shapes_its_keys(attribute: &Attribute) -> bool {
        attribute.path().is_ident("serde")
            && attribute
                .meta
                .require_list()
                .is_ok_and(|list| list.tokens.clone().into_iter().any(Self::is_key_shape))
    }

    fn is_key_shape(token: TokenTree) -> bool {
        matches!(token, TokenTree::Ident(ident) if Self::KEY_SHAPES.iter().any(|shape| ident == shape))
    }
}

#[derive(Default)]
struct FieldTypes(Vec<Name>);

impl<'ast> Visit<'ast> for FieldTypes {
    fn visit_type_path(&mut self, path: &'ast TypePath) {
        self.0.extend(
            path.path
                .segments
                .last()
                .map(|segment| Name::from(&segment.ident)),
        );
        syn::visit::visit_type_path(self, path);
    }
}
