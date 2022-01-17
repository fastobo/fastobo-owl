use fastobo::ast as obo;
use fastobo::ast::DateTime;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::datatype;

macro_rules! date_impl {
    ($type:ty) => {
        impl IntoOwlCtx for $type {
            type Owl = owl::Literal;
            fn into_owl(self, ctx: &mut Context) -> Self::Owl {
                owl::Literal::Datatype {
                    datatype_iri: ctx.build.iri(datatype::xsd::DATETIME),
                    literal: self.to_xsd_datetime(),
                }
            }
        }
    };
}

date_impl!(obo::NaiveDateTime);
date_impl!(obo::IsoDateTime);
