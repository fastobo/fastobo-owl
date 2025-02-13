use std::collections::BTreeSet;

use fastobo::ast as obo;
use horned_owl::model as owl;
use horned_owl::model::ForIRI;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::property;

// FIXME: Xrefs should probably be translated as IRIs instead of literals now
//        that Xrefs IDs have been formalized, but without an xref catalog
//        it is likely IRI expansion will be faulty.
impl<A: ForIRI> IntoOwlCtx<A> for obo::Xref {
    type Owl = owl::Annotation<A>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        owl::Annotation {
            ap: ctx
                .build
                .annotation_property(property::obo_in_owl::HAS_DBXREF),
            av: owl::AnnotationValue::Literal(owl::Literal::Simple {
                literal: self.id().to_string(),
            }),
        }
    }
}

impl<A: ForIRI> IntoOwlCtx<A> for obo::XrefList {
    type Owl = BTreeSet<<obo::Xref as IntoOwlCtx<A>>::Owl>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        self.into_iter().map(|xref| xref.into_owl(ctx)).collect()
    }
}
