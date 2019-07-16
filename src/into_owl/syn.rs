use std::collections::BTreeSet;
use std::iter::FromIterator;

use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::datatype;
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
            annotation_subject: ctx.current_frame.clone(),
            annotation: owl::Annotation {
                annotation_property: ctx.build.annotation_property(uri),
                annotation_value: d.into_owl(ctx).into(),
            },
        };

        let mut annotations =
            std::mem::replace(self.xrefs_mut(), obo::XrefList::default()).into_owl(ctx);
        if let Some(ty) = self.ty() {
            annotations.insert(owl::Annotation {
                annotation_property: ctx
                    .build
                    .annotation_property(property::obo_in_owl::HAS_SYNONYM_TYPE),
                annotation_value: owl::AnnotationValue::IRI(
                    obo::Ident::from(ty.clone()).into_owl(ctx),
                ),
            });
        }

        owl::AnnotatedAxiom::new(axiom, annotations)
    }
}
