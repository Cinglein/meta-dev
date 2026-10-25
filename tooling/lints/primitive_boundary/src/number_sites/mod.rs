mod sighting;
mod use_aliases;

use std::path::PathBuf;

use sighting::Sighting;
use use_aliases::UseAliases;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Block, ConstParam, Field, Fields, ImplItemConst, ItemConst, ItemEnum, ItemImpl, ItemMod,
    ItemStruct, ItemType, Signature, TraitItemConst, TypePath,
};

use super::finding::Finding;
use task::LineNumber;
use task::Policy;
use task::Site;
use task::SourceFile;
use task::Violation;
use task::Walk;
use task::WireType;
use task::{LoweredInfix, Name, Whole};

pub struct NumberSites<'scan> {
    path: PathBuf,
    policy: &'scan Policy,
    aliases: UseAliases,
    context: Name,
    newtypes: Vec<Name>,
    numbers: Vec<Violation>,
    sizes: Vec<Violation>,
}

impl<'scan> NumberSites<'scan> {
    const PRIMITIVES: [&'static str; 12] = [
        "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64",
    ];
    const SIZES: [&'static str; 2] = ["usize", "isize"];
    const SIZE_WORDS: [&'static str; 6] = ["length", "len", "capacity", "size", "bytes", "index"];

    pub fn violations(file: &SourceFile, policy: &'scan Policy) -> Vec<Violation> {
        if policy.bit_boundary().covers(file.path()) {
            Vec::new()
        } else {
            let mut sites = NumberSites {
                path: file.path().to_owned(),
                policy,
                aliases: UseAliases::in_file(file.syntax()),
                context: Name::default(),
                newtypes: Vec::new(),
                numbers: Vec::new(),
                sizes: Vec::new(),
            };
            sites.visit_file(file.syntax());
            if sites.newtypes.is_empty() {
                sites.numbers.into_iter().chain(sites.sizes).collect()
            } else {
                Vec::new()
            }
        }
    }

    fn is_named(path: &TypePath, names: &[&str]) -> bool {
        path.qself.is_none()
            && path
                .path
                .get_ident()
                .is_some_and(|ident| Name::from(ident).matches::<Whole>(names))
    }

    fn newtype(&self, item: &ItemStruct) -> Option<Name> {
        match &item.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => fields
                .unnamed
                .first()
                .filter(|field| matches!(&field.ty, syn::Type::Path(path) if Self::is_named(path, &Self::PRIMITIVES) || Self::is_named(path, &Self::SIZES) || self.policy.lists::<WireType>(&self.aliases.resolve(&path.path))))
                .map(|_| Name::from(&item.ident)),
            _ => None,
        }
    }

    fn sighting(&self, path: &TypePath) -> Sighting {
        let wire = self.aliases.resolve(&path.path);
        let site = Site::Line(self.path.clone(), LineNumber::from(path.path.span()));
        if Self::is_named(path, &Self::PRIMITIVES) {
            Sighting::Number(Violation::new(
                site,
                Finding::BarePrimitive {
                    primitive: path.path.get_ident().map(Name::from).unwrap_or_default(),
                    context: self.context.clone(),
                },
            ))
        } else if self.policy.lists::<WireType>(&wire) {
            Sighting::Number(Violation::new(
                site,
                Finding::WireForm {
                    wire,
                    context: self.context.clone(),
                },
            ))
        } else if Self::is_named(path, &Self::SIZES)
            && !self.context.matches::<LoweredInfix>(&Self::SIZE_WORDS)
        {
            Sighting::Size(Violation::new(
                site,
                Finding::UnnamedSize {
                    context: self.context.clone(),
                },
            ))
        } else {
            Sighting::Nothing
        }
    }
}

impl<'ast> Visit<'ast> for NumberSites<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match SourceFile::walk(module) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => syn::visit::visit_item_impl(self, item),
            Walk::Skipped => {}
        }
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.newtypes.extend(self.newtype(item));
        self.context = Name::from(&item.ident);
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.context = Name::from(&item.ident);
        syn::visit::visit_item_enum(self, item);
    }

    fn visit_const_param(&mut self, _: &'ast ConstParam) {}

    fn visit_block(&mut self, _: &'ast Block) {}

    fn visit_field(&mut self, field: &'ast Field) {
        self.context = field
            .ident
            .as_ref()
            .map_or_else(|| self.context.clone(), Name::from);
        syn::visit::visit_type(self, &field.ty);
    }

    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.context = Name::from(&item.ident);
        syn::visit::visit_type(self, &item.ty);
    }

    fn visit_impl_item_const(&mut self, item: &'ast ImplItemConst) {
        self.context = Name::from(&item.ident);
        syn::visit::visit_type(self, &item.ty);
    }

    fn visit_trait_item_const(&mut self, item: &'ast TraitItemConst) {
        self.context = Name::from(&item.ident);
        syn::visit::visit_type(self, &item.ty);
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.context = Name::from(&item.ident);
        syn::visit::visit_type(self, &item.ty);
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        self.context = Name::from(&signature.ident);
        syn::visit::visit_signature(self, signature);
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        self.sighting(path)
            .filed(&mut self.numbers, &mut self.sizes);
        syn::visit::visit_type_path(self, path);
    }
}
