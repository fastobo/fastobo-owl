use fastobo::ast as obo;
use horned_owl::model as owl;
use horned_owl::model::ForIRI;

use super::Context;
use super::IntoOwlCtx;

/// Convert a `PrefixedIdent` to an IRI using its IDspace or a default one.
impl<A: ForIRI> IntoOwlCtx<A> for &obo::PrefixedIdent {
    type Owl = owl::IRI<A>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        let iri = match ctx.idspaces.get(self.prefix()) {
            Some(url) => format!("{}{}", url, self.local()),
            None => format!(
                "{}{}_{}",
                crate::constants::uri::OBO,
                self.prefix(),
                self.local(),
            ),
        };
        ctx.build.iri(iri)
    }
}

/// Convert an `UnprefixedIdent` to an OWL IRI using the ontology IRI.
impl<A: ForIRI> IntoOwlCtx<A> for &obo::UnprefixedIdent {
    type Owl = owl::IRI<A>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        ctx.build
            .iri(format!("{}#{}", &ctx.ontology_iri, self.as_str()))
    }
}

/// Convert an OBO URL identifier to an OWL IRI.
impl<A: ForIRI> IntoOwlCtx<A> for &obo::Url {
    type Owl = owl::IRI<A>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        ctx.build.iri(self.as_str())
    }
}

/// Convert an arbitrary OBO identifier to an OWL IRI.
impl<A: ForIRI> IntoOwlCtx<A> for &obo::Ident {
    type Owl = owl::IRI<A>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        match self {
            obo::Ident::Url(url) => url.into_owl(ctx),
            obo::Ident::Unprefixed(id) => id.into_owl(ctx),
            obo::Ident::Prefixed(id) => id.into_owl(ctx),
        }
    }
}

/// Convert a class identifier to an OWL IRI.
impl<A: ForIRI> IntoOwlCtx<A> for &obo::ClassIdent {
    type Owl = owl::IRI<A>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        self.as_ref().into_owl(ctx)
    }
}

/// Convert a subset identifier to an OWL IRI.
// FIXME: this is context-dependent! The IRI replacement rule must be used
//        if the typedef is just a local unprefixed alias for an imported
//        typedef.
impl<A: ForIRI> IntoOwlCtx<A> for &obo::RelationIdent {
    type Owl = owl::IRI<A>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        self.as_ref().into_owl(ctx)
    }
}

/// Convert a subset identifier to an OWL IRI.
impl<A: ForIRI> IntoOwlCtx<A> for &obo::SubsetIdent {
    type Owl = owl::IRI<A>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        self.as_ref().into_owl(ctx)
    }
}

/// Convert a subset identifier to an OWL IRI.
impl<A: ForIRI> IntoOwlCtx<A> for &obo::SynonymTypeIdent {
    type Owl = owl::IRI<A>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        self.as_ref().into_owl(ctx)
    }
}
