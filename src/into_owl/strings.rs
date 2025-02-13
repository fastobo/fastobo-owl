use fastobo::ast as obo;
use horned_owl::model as owl;
use horned_owl::model::ForIRI;

use super::Context;
use super::IntoOwlCtx;

macro_rules! string_impl {
    ($type:ty) => {
        impl<A: ForIRI> IntoOwlCtx<A> for $type {
            type Owl = owl::Literal<A>;
            fn into_owl(self, _ctx: &mut Context<A>) -> Self::Owl {
                owl::Literal::Simple {
                    literal: self.into_string(),
                }
            }
        }
    };
}

string_impl!(obo::QuotedString);
string_impl!(obo::UnquotedString);
