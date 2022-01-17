use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::property;

impl IntoOwlCtx for obo::Synonym {
    type Owl = owl::AnnotatedAxiom;
    fn into_owl(mut self, ctx: &mut Context) -> Self::Owl {
        // Get the annotation property URI
        let uri = match self.scope() {
            obo::SynonymScope::Exact => property::obo_in_owl::HAS_EXACT_SYNONYM,
            obo::SynonymScope::Narrow => property::obo_in_owl::HAS_NARROW_SYNONYM,
            obo::SynonymScope::Related => property::obo_in_owl::HAS_RELATED_SYNONYM,
            obo::SynonymScope::Broad => property::obo_in_owl::HAS_BROAD_SYNONYM,
        };

        // Get the description
        let d = std::mem::replace(
            self.description_mut(),
            obo::QuotedString::new(String::new()),
        );

        let axiom = owl::AnnotationAssertion {
            subject: owl::Individual::from(&ctx.current_frame),
            ann: owl::Annotation {
                ap: ctx.build.annotation_property(uri),
                av: d.into_owl(ctx).into(),
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

        owl::AnnotatedAxiom::new(axiom, annotations)
    }
}
