use fastobo::ast as obo;
use horned_owl::model as owl;
use horned_owl::model::ForIRI;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::datatype::xsd;

fn is_xsd_string<A: ForIRI>(ctx: &Context<A>, id: &obo::Ident) -> bool {
    match id {
        obo::Ident::Unprefixed(_) => false,
        obo::Ident::Url(url) => url.as_str() == xsd::STRING,
        obo::Ident::Prefixed(pid) => match ctx.idspaces.get(pid.prefix()) {
            None => pid.prefix() == "xsd" && pid.local() == "string",
            Some(base_url) => {
                let url = format!("{}{}", base_url, pid.local());
                url == xsd::STRING
            }
        },
    }
}

impl<A: ForIRI> IntoOwlCtx<A> for obo::PropertyValue {
    type Owl = owl::Annotation<A>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        match self {
            obo::PropertyValue::Resource(pv) => owl::Annotation {
                ap: owl::AnnotationProperty(pv.property().into_owl(ctx)),
                av: owl::AnnotationValue::IRI(pv.target().into_owl(ctx)),
            },
            obo::PropertyValue::Literal(pv) => owl::Annotation {
                ap: owl::AnnotationProperty(pv.property().into_owl(ctx)),
                av: owl::AnnotationValue::Literal(if is_xsd_string(&ctx, pv.datatype()) {
                    owl::Literal::Simple {
                        literal: pv.literal().as_str().to_string(),
                    }
                } else {
                    owl::Literal::Datatype {
                        datatype_iri: pv.datatype().into_owl(ctx),
                        literal: pv.literal().as_str().to_string(),
                    }
                }),
            },
        }
    }
}
