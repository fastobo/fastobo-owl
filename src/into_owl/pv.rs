use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;

impl IntoOwlCtx for obo::PropertyValue {
    type Owl = owl::Annotation;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        match self {
            obo::PropertyValue::Resource(rel, id) => owl::Annotation {
                ap: owl::AnnotationProperty(obo::Ident::from(rel).into_owl(ctx)),
                av: owl::AnnotationValue::IRI(id.into_owl(ctx)),
            },
            obo::PropertyValue::Literal(rel, value, dty) => owl::Annotation {
                ap: owl::AnnotationProperty(obo::Ident::from(rel).into_owl(ctx)),
                av: owl::AnnotationValue::Literal(owl::Literal {
                    datatype_iri: Some(dty.into_owl(ctx)),
                    literal: Some(value.into_string()),
                    lang: None,
                }),
            },
        }
    }
}
