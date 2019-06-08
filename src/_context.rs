use std::collections::HashMap;
use std::collections::HashSet;

use fastobo::ast as obo;
use horned_owl::model as owl;

lazy_static! {
    static ref CARDINALITY: obo::RelationIdent =
        obo::RelationIdent::from(obo::UnprefixedIdent::new("cardinality"));
    static ref MIN_CARDINALITY: obo::RelationIdent =
        obo::RelationIdent::from(obo::UnprefixedIdent::new("minCardinality"));
    static ref MAX_CARDINALITY: obo::RelationIdent =
        obo::RelationIdent::from(obo::UnprefixedIdent::new("maxCardinality"));
    static ref ALL_ONLY: obo::RelationIdent =
        obo::RelationIdent::from(obo::UnprefixedIdent::new("all_only"));
    static ref ALL_SOME: obo::RelationIdent =
        obo::RelationIdent::from(obo::UnprefixedIdent::new("all_some"));
}

/// An opaque structure to pass context arguments required for OWL conversion.
pub struct Context {
    ///
    pub build: owl::Build,

    // prefixes: curie::PrefixMapping,
    pub idspaces: HashMap<obo::IdentPrefix, obo::Url>,

    pub ontology_iri: obo::Url,

    pub current_frame: owl::IRI,

    /// A set of IRI which refer to class level annotation relationships.
    ///
    /// This is likely to require processing imports beforehand.
    pub class_level: HashSet<owl::IRI>,
}

impl Context {
    pub fn is_class_level(&mut self, rid: &owl::IRI) -> bool {
        self.class_level.contains(&rid)
    }

    pub fn rel_class_expression(
        &mut self,
        qualifiers: &obo::QualifierList,
        relation: owl::ObjectProperty,
        cls: Box<owl::ClassExpression>,
    ) -> owl::ClassExpression {
        if let Some(q) = qualifiers.iter().find(|q| q.key() == &*CARDINALITY) {
            let n: i32 = q.value().parse().expect("invalid value for `cardinality`");
            if n == 0 {
                return owl::ClassExpression::ObjectAllValuesFrom {
                    o: owl::ObjectPropertyExpression::ObjectProperty(relation),
                    ce: Box::new(owl::ClassExpression::ObjectComplementOf { ce: cls }),
                };
            } else {
                return owl::ClassExpression::ObjectExactCardinality {
                    n,
                    o: owl::ObjectPropertyExpression::ObjectProperty(relation),
                    ce: cls,
                };
            }
        }

        if let Some(q) = qualifiers.iter().find(|q| q.key() == &*MAX_CARDINALITY) {
            let na: i32 = q
                .value()
                .parse()
                .expect("invalid value for `maxCardinality`");
            if na == 0 {
                return owl::ClassExpression::ObjectAllValuesFrom {
                    o: owl::ObjectPropertyExpression::ObjectProperty(relation),
                    ce: Box::new(owl::ClassExpression::ObjectComplementOf { ce: cls }),
                };
            }
        }

        if let Some(qa) = qualifiers.iter().find(|q| q.key() == &*MIN_CARDINALITY) {
            let na = qa
                .value()
                .parse()
                .expect("invalid value for `min_cardinality`");
            if let Some(qb) = qualifiers.iter().find(|q| q.key() == &*MAX_CARDINALITY) {
                let nb = qb
                    .value()
                    .parse()
                    .expect("invalid value for `max_cardinality`");;
                return owl::ClassExpression::ObjectIntersectionOf {
                    o: vec![
                        owl::ClassExpression::ObjectMinCardinality {
                            n: na,
                            o: owl::ObjectPropertyExpression::ObjectProperty(relation.clone()),
                            ce: cls.clone(),
                        },
                        owl::ClassExpression::ObjectMaxCardinality {
                            n: nb,
                            o: owl::ObjectPropertyExpression::ObjectProperty(relation),
                            ce: cls,
                        },
                    ],
                };
            } else {
                return owl::ClassExpression::ObjectMinCardinality {
                    n: na,
                    o: owl::ObjectPropertyExpression::ObjectProperty(relation),
                    ce: cls,
                };
            }
        }

        if let Some(q) = qualifiers.iter().find(|q| q.key() == &*MAX_CARDINALITY) {
            return owl::ClassExpression::ObjectMaxCardinality {
                n: q.value()
                    .parse()
                    .expect("invalid value for `maxCardinality`"),
                o: owl::ObjectPropertyExpression::ObjectProperty(relation),
                ce: Box::new(owl::ClassExpression::ObjectComplementOf { ce: cls }),
            };
        }

        if let Some(q_only) = qualifiers.iter().find(|q| q.key() == &*ALL_ONLY) {
            if let Some(q_some) = qualifiers.iter().find(|q| q.key() == &*ALL_SOME) {
                return owl::ClassExpression::ObjectIntersectionOf {
                    o: vec![
                        owl::ClassExpression::ObjectSomeValuesFrom {
                            o: owl::ObjectPropertyExpression::ObjectProperty(relation.clone()),
                            ce: cls.clone(),
                        },
                        owl::ClassExpression::ObjectAllValuesFrom {
                            o: owl::ObjectPropertyExpression::ObjectProperty(relation),
                            ce: cls,
                        },
                    ],
                };
            } else {
                return owl::ClassExpression::ObjectAllValuesFrom {
                    o: owl::ObjectPropertyExpression::ObjectProperty(relation),
                    ce: cls,
                };
            }
        }

        // FIXME: is_class_level
        if self.is_class_level(&relation.0) {
            owl::ClassExpression::ObjectHasValue {
                o: owl::ObjectPropertyExpression::ObjectProperty(relation),
                i: match *cls {
                    owl::ClassExpression::Class(c) => owl::NamedIndividual(c.0),
                    _ => unreachable!(),
                },
            }
        } else {
            owl::ClassExpression::ObjectSomeValuesFrom {
                o: owl::ObjectPropertyExpression::ObjectProperty(relation),
                ce: cls,
            }
        }
    }
}
