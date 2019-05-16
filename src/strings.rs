use fastobo::ast as obo;
use horned_owl::model as owl;

use crate::constants::datatype;
use super::Context;
use super::IntoOwlCtx;

impl IntoOwlCtx for obo::QuotedString {
    type Owl = owl::Literal;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        owl::Literal {
            datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
            literal: Some(self.into_string()),
            lang: None,
        }
    }
}

impl IntoOwlCtx for obo::UnquotedString {
    type Owl = owl::Literal;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        owl::Literal {
            datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
            literal: Some(self.into_string()),
            lang: None,
        }
    }
}
