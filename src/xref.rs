use std::collections::BTreeSet;

use fastobo::ast as obo;
use horned_owl::model as owl;

use crate::constants::datatype;
use crate::constants::property;
use super::Context;
use super::IntoOwlCtx;

impl IntoOwlCtx for obo::Xref {
    type Owl = owl::Annotation;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        owl::Annotation {
            annotation_property: ctx.build.annotation_property(property::obo_in_owl::XREF),
            annotation_value: owl::AnnotationValue::Literal(
                owl::Literal {
                    lang: None,
                    datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                    literal: Some(self.id().to_string())
                }
            )
        }
    }
}

impl IntoOwlCtx for obo::XrefList {
    type Owl = BTreeSet<<obo::Xref as IntoOwlCtx>::Owl>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        self.into_iter().map(|xref| xref.into_owl(ctx)).collect()
    }
}
