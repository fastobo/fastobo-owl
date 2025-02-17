use fastobo::ast as obo;
use horned_owl::model as owl;
use horned_owl::model::ForIRI;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::property;

impl<A: ForIRI> IntoOwlCtx<A> for obo::Synonym {
    type Owl = owl::AnnotatedComponent<A>;
    fn into_owl(mut self, ctx: &mut Context<A>) -> Self::Owl {
        // Build the main assertion
        let axiom = owl::AnnotationAssertion {
            subject: owl::AnnotationSubject::from(&ctx.current_frame),
            ann: owl::Annotation {
                ap: owl::AnnotationProperty::from(self.scope().into_owl(ctx)),
                av: std::mem::take(self.description_mut()).into_owl(ctx).into(),
            },
        };

        let mut annotations =
            std::mem::replace(self.xrefs_mut(), obo::XrefList::default()).into_owl(ctx);
        if let Some(ty) = self.ty() {
            annotations.insert(owl::Annotation {
                ap: ctx
                    .build
                    .annotation_property(property::obo_in_owl::HAS_SYNONYM_TYPE),
                av: owl::AnnotationValue::IRI(ty.into_owl(ctx)),
            });
        }

        owl::AnnotatedComponent::new(axiom, annotations)
    }
}

impl<A: ForIRI> IntoOwlCtx<A> for &obo::SynonymScope {
    type Owl = owl::IRI<A>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        ctx.build.iri(match self {
            obo::SynonymScope::Broad => property::obo_in_owl::HAS_BROAD_SYNONYM,
            obo::SynonymScope::Exact => property::obo_in_owl::HAS_EXACT_SYNONYM,
            obo::SynonymScope::Narrow => property::obo_in_owl::HAS_NARROW_SYNONYM,
            obo::SynonymScope::Related => property::obo_in_owl::HAS_RELATED_SYNONYM,
        })
    }
}
