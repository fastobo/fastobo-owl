use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::property;

impl IntoOwlCtx for obo::Definition {
    type Owl = owl::AnnotatedAxiom;
    fn into_owl(mut self, ctx: &mut Context) -> Self::Owl {
        let xrefs = std::mem::take(self.xrefs_mut());
        owl::AnnotatedAxiom::new(
            owl::AnnotationAssertion::new(
                owl::Individual::from(&ctx.current_frame),
                owl::Annotation {
                    ap: ctx.build.annotation_property(property::iao::DEFINITION),
                    av: owl::AnnotationValue::Literal(owl::Literal::Simple {
                        literal: self.text().as_str().to_string(),
                    }),
                },
            ),
            xrefs.into_owl(ctx),
        )
    }
}
