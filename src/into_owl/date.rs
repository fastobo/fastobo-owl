use fastobo::ast as obo;
use fastobo::ast::Date;
use fastobo::ast::DateTime;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::datatype::xsd;

macro_rules! datetime_impl {
    ($type:ty) => {
        impl IntoOwlCtx for &$type {
            type Owl = owl::Literal;
            fn into_owl(self, ctx: &mut Context) -> Self::Owl {
                owl::Literal::Datatype {
                    literal: self.to_xsd_datetime(),
                    datatype_iri: ctx.build.iri(xsd::DATETIME),
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

datetime_impl!(obo::NaiveDateTime);
datetime_impl!(obo::IsoDateTime);

impl IntoOwlCtx for &obo::IsoDate {
    type Owl = owl::Literal;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        owl::Literal::Datatype {
            literal: self.to_xsd_date(),
            datatype_iri: ctx.build.iri(xsd::DATE),
        }
    }
}

impl IntoOwlCtx for obo::IsoDate {
    type Owl = owl::Literal;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        (&self).into_owl(ctx)
    }
}

impl IntoOwlCtx for &obo::CreationDate {
    type Owl = owl::Literal;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        match self {
            obo::CreationDate::Date(d) => <&obo::IsoDate as IntoOwlCtx>::into_owl(&d, ctx),
            obo::CreationDate::DateTime(dt) => {
                <&obo::IsoDateTime as IntoOwlCtx>::into_owl(&dt, ctx)
            }
        }
    }
}

impl IntoOwlCtx for obo::CreationDate {
    type Owl = owl::Literal;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        (&self).into_owl(ctx)
    }
}
