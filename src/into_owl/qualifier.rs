use std::collections::BTreeSet;

use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;

lazy_static! {
    static ref EXCLUDED: BTreeSet<obo::RelationIdent> = {
        let mut s = BTreeSet::new();
        s.insert(obo::RelationIdent::from(obo::UnprefixedIdent::new(
            "cardinality",
        )));
        s.insert(obo::RelationIdent::from(obo::UnprefixedIdent::new(
            "minCardinality",
        )));
        s.insert(obo::RelationIdent::from(obo::UnprefixedIdent::new(
            "maxCardinality",
        )));
        s.insert(obo::RelationIdent::from(obo::UnprefixedIdent::new(
            "gci_relation",
        )));
        s.insert(obo::RelationIdent::from(obo::UnprefixedIdent::new(
            "gci_filler",
        )));
        s.insert(obo::RelationIdent::from(obo::UnprefixedIdent::new(
            "all_some",
        )));
        s.insert(obo::RelationIdent::from(obo::UnprefixedIdent::new(
            "all_only",
        )));
        s
    };
}

impl IntoOwlCtx for obo::Qualifier {
    type Owl = Option<owl::Annotation>;
    fn into_owl(mut self, ctx: &mut Context) -> Self::Owl {
        if !EXCLUDED.contains(self.key()) {
            // Take ownership of key and value without extra heap allocation.
            let key = std::mem::replace(
                self.key_mut(),
                obo::UnprefixedIdent::new(String::new()).into(),
            );
            let value = std::mem::take(self.value_mut());
            // Build the annotation.
            Some(owl::Annotation {
                ap: key.into_owl(ctx).into(),
                av: owl::AnnotationValue::Literal(owl::Literal::Simple {
                    literal: value.as_str().to_string(),
                }),
            })
        } else {
            None
        }
    }
}

impl IntoOwlCtx for obo::QualifierList {
    type Owl = BTreeSet<owl::Annotation>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        self.into_iter().flat_map(|q| q.into_owl(ctx)).collect()
    }
}
