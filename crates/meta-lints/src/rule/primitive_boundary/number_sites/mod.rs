mod bare_number;
mod unnamed_size;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Block, ConstParam, Field, Fields, Ident, ImplItemConst, ImplItemFn, ItemConst, ItemEnum, ItemImpl, ItemMod,
    ItemStruct, ItemType, Signature, TraitItemConst, TypePath,
};

use bare_number::BareNumber;
use unnamed_size::UnnamedSize;

use crate::region::Region;
use crate::rule::name::Name;
use crate::rule::violation::Violation;
use crate::site::Site;
use crate::source_file::SourceFile;

pub struct NumberSites {
    path: String,
    context: Option<Name>,
    newtypes: Vec<Name>,
    numbers: Vec<Violation>,
    sizes: Vec<Violation>,
}

impl NumberSites {
    const PRIMITIVES: [&str; 12] = ["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64"];
    const SIZES: [&str; 2] = ["usize", "isize"];
    const SIZE_WORDS: [&str; 6] = ["length", "len", "capacity", "size", "bytes", "index"];

    pub fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut sites = NumberSites {
            path: file.path().to_owned(),
            context: None,
            newtypes: Vec::new(),
            numbers: Vec::new(),
            sizes: Vec::new(),
        };
        sites.visit_file(file.syntax());
        if file.region() == Region::Boundary || !sites.newtypes.is_empty() {
            Vec::new()
        } else if file.region() == Region::Product {
            sites.numbers.into_iter().chain(sites.sizes).collect()
        } else {
            sites.numbers
        }
    }

    fn named<'path>(path: &'path TypePath, names: &[&str]) -> Option<&'path Ident> {
        path.path.get_ident().filter(|ident| path.qself.is_none() && names.iter().any(|name| *ident == name))
    }

    fn wraps_primitive(item: &ItemStruct) -> bool {
        match &item.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => fields.unnamed.first().is_some_and(|field| {
                matches!(&field.ty, syn::Type::Path(path) if Self::named(path, &Self::PRIMITIVES).or(Self::named(path, &Self::SIZES)).is_some())
            }),
            _ => false,
        }
    }

    fn names_a_size(&self) -> bool {
        let lowered = self.context.as_ref().map(Name::as_str).unwrap_or_default().to_lowercase();
        Self::SIZE_WORDS.iter().any(|word| lowered.contains(word))
    }

    fn site(&self, path: &TypePath) -> Site {
        Site::spanned(&self.path, path.span())
    }
}

impl<'ast> Visit<'ast> for NumberSites {
    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        if !SourceFile::is_test_item(&function.attrs) {
            syn::visit::visit_impl_item_fn(self, function);
        }
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !SourceFile::is_test_item(&module.attrs) {
            syn::visit::visit_item_mod(self, module);
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        if item.trait_.is_none() {
            syn::visit::visit_item_impl(self, item);
        }
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.newtypes.extend(Self::wraps_primitive(item).then(|| Name::from(&item.ident)));
        self.context = Some(Name::from(&item.ident));
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.context = Some(Name::from(&item.ident));
        syn::visit::visit_item_enum(self, item);
    }

    fn visit_const_param(&mut self, _: &'ast ConstParam) {}

    fn visit_block(&mut self, _: &'ast Block) {}

    fn visit_field(&mut self, field: &'ast Field) {
        self.context = field.ident.as_ref().map(Name::from).or_else(|| self.context.take());
        syn::visit::visit_type(self, &field.ty);
    }

    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.context = Some(Name::from(&item.ident));
        syn::visit::visit_type(self, &item.ty);
    }

    fn visit_impl_item_const(&mut self, item: &'ast ImplItemConst) {
        self.context = Some(Name::from(&item.ident));
        syn::visit::visit_type(self, &item.ty);
    }

    fn visit_trait_item_const(&mut self, item: &'ast TraitItemConst) {
        self.context = Some(Name::from(&item.ident));
        syn::visit::visit_type(self, &item.ty);
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.context = Some(Name::from(&item.ident));
        syn::visit::visit_type(self, &item.ty);
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        self.context = Some(Name::from(&signature.ident));
        syn::visit::visit_signature(self, signature);
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        let bare = Self::named(path, &Self::PRIMITIVES).map(|primitive| {
            Violation::new(self.site(path), BareNumber::new(Name::from(primitive), self.context.clone()))
        });
        let unnamed = Self::named(path, &Self::SIZES)
            .filter(|_| !self.names_a_size())
            .map(|_| Violation::new(self.site(path), UnnamedSize::new(self.context.clone())));
        self.numbers.extend(bare);
        self.sizes.extend(unnamed);
        syn::visit::visit_type_path(self, path);
    }
}
