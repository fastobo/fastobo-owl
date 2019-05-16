use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::datatype;

macro_rules! string_impl {
    ($type:ty) => {
        impl IntoOwlCtx for $type {
            type Owl = owl::Literal;
            fn into_owl(self, ctx: &mut Context) -> Self::Owl {
                owl::Literal {
                    datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                    literal: Some(self.into_string()),
                    lang: None,
                }
            }
        }
    };
}

string_impl!(obo::QuotedString);
string_impl!(obo::UnquotedString);
