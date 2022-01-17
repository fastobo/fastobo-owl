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
                owl::Literal::Datatype {
                    datatype_iri: ctx.build.iri(datatype::xsd::STRING),
                    literal: self.into_string(),
                }
            }
        }
    };
}

string_impl!(obo::QuotedString);
string_impl!(obo::UnquotedString);
