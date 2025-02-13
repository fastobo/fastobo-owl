use fastobo::ast as obo;
use horned_owl::model as owl;
use horned_owl::model::ForIRI;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::property;

impl<A: ForIRI> IntoOwlCtx<A> for obo::Definition {
    type Owl = owl::AnnotatedComponent<A>;
    fn into_owl(mut self, ctx: &mut Context<A>) -> Self::Owl {
        let xrefs = std::mem::take(self.xrefs_mut());
        owl::AnnotatedComponent::new(
            owl::AnnotationAssertion::new(
                owl::AnnotationSubject::from(&ctx.current_frame),
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
