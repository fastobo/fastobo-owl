use fastobo::ast as obo;
use fastobo::ast::DateTime;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;

macro_rules! date_impl {
    ($type:ty) => {
        impl IntoOwlCtx for &$type {
            type Owl = owl::Literal;
            fn into_owl(self, _ctx: &mut Context) -> Self::Owl {
                owl::Literal::Simple {
                    literal: self.to_xsd_datetime(),
                }
            }
        }

        impl IntoOwlCtx for $type {
            type Owl = owl::Literal;
            fn into_owl(self, ctx: &mut Context) -> Self::Owl {
                (&self).into_owl(ctx)
            }
        }
    };
}

date_impl!(obo::NaiveDateTime);
date_impl!(obo::IsoDateTime);
