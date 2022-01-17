use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;

macro_rules! string_impl {
    ($type:ty) => {
        impl IntoOwlCtx for $type {
            type Owl = owl::Literal;
            fn into_owl(self, _ctx: &mut Context) -> Self::Owl {
                owl::Literal::Simple {
                    literal: self.into_string(),
                }
            }
        }
    };
}

string_impl!(obo::QuotedString);
string_impl!(obo::UnquotedString);
