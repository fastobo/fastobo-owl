use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;

impl IntoOwlCtx for obo::PropertyValue {
    type Owl = owl::Annotation;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        match self {
            obo::PropertyValue::Resource(pv) => owl::Annotation {
                ap: owl::AnnotationProperty(pv.property().into_owl(ctx)),
                av: owl::AnnotationValue::IRI(pv.target().into_owl(ctx)),
            },
            obo::PropertyValue::Literal(pv) => owl::Annotation {
                ap: owl::AnnotationProperty(pv.property().into_owl(ctx)),
                av: owl::AnnotationValue::Literal(owl::Literal::Datatype {
                    datatype_iri: pv.datatype().into_owl(ctx),
                    literal: pv.literal().to_string(),
                }),
            },
        }
    }
}
