use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;

/// Convert a `PrefixedIdent` to an IRI using its IDspace or a default one.
impl IntoOwlCtx for &obo::PrefixedIdent {
    type Owl = owl::IRI;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
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
impl IntoOwlCtx for &obo::UnprefixedIdent {
    type Owl = owl::IRI;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        ctx.build
            .iri(format!("{}#{}", &ctx.ontology_iri, self.as_str()))
    }
}

/// Convert an OBO URL identifier to an OWL IRI.
impl IntoOwlCtx for &obo::Url {
    type Owl = owl::IRI;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        ctx.build.iri(self.as_str())
    }
}

/// Convert an arbitrary OBO identifier to an OWL IRI.
impl IntoOwlCtx for &obo::Ident {
    type Owl = owl::IRI;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        match self {
            obo::Ident::Url(url) => url.into_owl(ctx),
            obo::Ident::Unprefixed(id) => id.into_owl(ctx),
            obo::Ident::Prefixed(id) => id.into_owl(ctx),
        }
    }
}

/// Convert a class identifier to an OWL IRI.
impl IntoOwlCtx for &obo::ClassIdent {
    type Owl = owl::IRI;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        self.as_ref().into_owl(ctx)
    }
}

/// Convert a subset identifier to an OWL IRI.
// FIXME: this is context-dependent! The IRI replacement rule must be used
//        if the typedef is just a local unprefixed alias for an imported
//        typedef.
impl IntoOwlCtx for &obo::RelationIdent {
    type Owl = owl::IRI;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        self.as_ref().into_owl(ctx)
    }
}

/// Convert a subset identifier to an OWL IRI.
impl IntoOwlCtx for &obo::SubsetIdent {
    type Owl = owl::IRI;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        self.as_ref().into_owl(ctx)
    }
}

/// Convert a subset identifier to an OWL IRI.
impl IntoOwlCtx for &obo::SynonymTypeIdent {
    type Owl = owl::IRI;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        self.as_ref().into_owl(ctx)
    }
}
