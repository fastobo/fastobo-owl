use std::collections::BTreeSet;

use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::datatype;
use crate::constants::property;

// FIXME: Xrefs should probably be translated as IRIs instead of literals now
//        that Xrefs IDs have been formalized, but without an xref catalog
//        it is likely IRI expansion will be faulty.
impl IntoOwlCtx for obo::Xref {
    type Owl = owl::Annotation;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        owl::Annotation {
            annotation_property: ctx
                .build
                .annotation_property(property::obo_in_owl::HAS_DBXREF),
            annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                lang: None,
                datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                literal: Some(self.id().to_string()),
            }),
        }
    }
}

impl IntoOwlCtx for obo::XrefList {
    type Owl = BTreeSet<<obo::Xref as IntoOwlCtx>::Owl>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        self.into_iter().map(|xref| xref.into_owl(ctx)).collect()
    }
}
