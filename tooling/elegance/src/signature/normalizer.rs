use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;
use syn::{
    AngleBracketedGenericArguments, GenericArgument, Ident, Path, PathArguments, PathSegment, Receiver, TypeReference,
};

pub struct Normalizer<'own> {
    own: Option<&'own Ident>,
}

impl<'own> Normalizer<'own> {
    const SELF: &'static str = "Self";

    #[must_use]
    pub fn new(own: Option<&'own Ident>) -> Normalizer<'own> {
        Normalizer { own }
    }

    fn abstracted(&self, segment: PathSegment) -> PathSegment {
        match self.own {
            Some(own) if segment.ident == *own => PathSegment::from(Ident::new(Self::SELF, Span::call_site())),
            _ => segment,
        }
    }
}

impl VisitMut for Normalizer<'_> {
    fn visit_path_mut(&mut self, path: &mut Path) {
        path.segments.iter_mut().for_each(|segment| self.visit_path_segment_mut(segment));
        *path = Path {
            leading_colon: None,
            segments: path
                .segments
                .last()
                .cloned()
                .map(|segment| self.abstracted(segment))
                .into_iter()
                .collect::<Punctuated<PathSegment, syn::token::PathSep>>(),
        };
    }

    fn visit_type_reference_mut(&mut self, reference: &mut TypeReference) {
        reference.lifetime.take();
        self.visit_type_mut(&mut reference.elem);
    }

    fn visit_receiver_mut(&mut self, receiver: &mut Receiver) {
        receiver.reference.as_mut().and_then(|reference| reference.1.take());
        self.visit_type_mut(&mut receiver.ty);
    }

    fn visit_path_arguments_mut(&mut self, arguments: &mut PathArguments) {
        *arguments = match arguments {
            PathArguments::AngleBracketed(angled) => Some(AngleBracketedGenericArguments {
                args: angled
                    .args
                    .iter()
                    .filter(|argument| !matches!(argument, GenericArgument::Lifetime(_)))
                    .cloned()
                    .collect(),
                ..angled.clone()
            })
            .filter(|angled| !angled.args.is_empty())
            .map_or(PathArguments::None, PathArguments::AngleBracketed),
            PathArguments::None | PathArguments::Parenthesized(_) => arguments.clone(),
        };
        syn::visit_mut::visit_path_arguments_mut(self, arguments);
    }
}
