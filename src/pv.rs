use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;

impl IntoOwlCtx for obo::PropertyValue {
    type Owl = owl::Annotation;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        match self {
            obo::PropertyValue::Identified(rel, id) => owl::Annotation {
                annotation_property: owl::AnnotationProperty(obo::Ident::from(rel).into_owl(ctx)),
                annotation_value: owl::AnnotationValue::IRI(id.into_owl(ctx))
            },
            obo::PropertyValue::Typed(rel, value, dty) => owl::Annotation {
                annotation_property: owl::AnnotationProperty(obo::Ident::from(rel).into_owl(ctx)),
                annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                    datatype_iri: Some(obo::Ident::from(dty).into_owl(ctx)),
                    literal: Some(value.into_string()),
                    lang: None,
                })
            }
        }
    }
}
