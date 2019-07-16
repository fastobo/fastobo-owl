mod date;
mod doc;
mod header;
mod id;
mod pv;
mod qualifier;
mod strings;
mod syn;
mod term;
mod xref;

use std::collections::HashMap;
use std::collections::HashSet;

use fastobo::ast as obo;
use horned_owl::model as owl;

// ---------------------------------------------------------------------------

/// The internal trait for data conversion;
///
/// This is not exposed because `ctx` can be mostly inferred from the source
/// OBO ontology, therefore a public trait shall be made available only for
/// the `OboDoc` struct, with less arguments to provide.
trait IntoOwlCtx {
    type Owl;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl;
}

/// The public trait for context-free OBO to OWL conversion.
pub trait IntoOwl {
    /// Get the CURIE prefix mapping using IDSpaces declared in the document.
    ///
    /// This lets prefixed identifiers be shortened back again as CURIEs
    /// in the OWL serialization. Default OBO prefixes are included (see
    /// [`obo_prefixes`](./fn.obo_prefixes.html)).
    fn prefixes(&self) -> curie::PrefixMapping;
    /// Convert the OBO document into an `Ontology` in OWL language.
    fn into_owl(self) -> owl::Ontology;
}

// ---------------------------------------------------------------------------

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
        relation: obo::RelationIdent,
        cls: obo::ClassIdent,
    ) -> owl::ClassExpression {
        let r_iri: owl::IRI = relation.into_owl(self);
        let c_iri: owl::IRI = cls.into_owl(self);

        if let Some(q) = qualifiers.iter().find(|q| q.key() == &*CARDINALITY) {
            let n: u32 = q.value().parse().expect("invalid value for `cardinality`");
            if n == 0 {
                return owl::ClassExpression::ObjectAllValuesFrom {
                    o: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(r_iri)),
                    ce: Box::new(owl::ClassExpression::ObjectComplementOf {
                        ce: Box::new(owl::Class(c_iri).into()),
                    }),
                };
            } else {
                return owl::ClassExpression::ObjectExactCardinality {
                    n,
                    o: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(r_iri)),
                    ce: Box::new(owl::Class(c_iri).into()),
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
                    o: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(r_iri)),
                    ce: Box::new(owl::ClassExpression::ObjectComplementOf {
                        ce: Box::new(owl::Class(c_iri).into()),
                    }),
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
                            o: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(
                                r_iri.clone(),
                            )),
                            ce: Box::new(owl::Class(c_iri.clone()).into()),
                        },
                        owl::ClassExpression::ObjectMaxCardinality {
                            n: nb,
                            o: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(
                                r_iri.clone(),
                            )),
                            ce: Box::new(owl::Class(c_iri.clone()).into()),
                        },
                    ],
                };
            } else {
                return owl::ClassExpression::ObjectMinCardinality {
                    n: na,
                    o: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(r_iri)),
                    ce: Box::new(owl::Class(c_iri).into()),
                };
            }
        }

        if let Some(q) = qualifiers.iter().find(|q| q.key() == &*MAX_CARDINALITY) {
            return owl::ClassExpression::ObjectMaxCardinality {
                n: q.value()
                    .parse()
                    .expect("invalid value for `maxCardinality`"),
                o: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(r_iri)),
                ce: Box::new(owl::ClassExpression::ObjectComplementOf {
                    ce: Box::new(owl::Class(c_iri).into()),
                }),
            };
        }

        if let Some(q_only) = qualifiers.iter().find(|q| q.key() == &*ALL_ONLY) {
            if let Some(q_some) = qualifiers.iter().find(|q| q.key() == &*ALL_SOME) {
                return owl::ClassExpression::ObjectIntersectionOf {
                    o: vec![
                        owl::ClassExpression::ObjectSomeValuesFrom {
                            o: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(
                                r_iri.clone(),
                            )),
                            ce: Box::new(owl::Class(c_iri.clone()).into()),
                        },
                        owl::ClassExpression::ObjectAllValuesFrom {
                            o: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(
                                r_iri,
                            )),
                            ce: Box::new(owl::Class(c_iri).into()),
                        },
                    ],
                };
            } else {
                return owl::ClassExpression::ObjectAllValuesFrom {
                    o: owl::ObjectPropertyExpression::ObjectProperty(r_iri.into()),
                    ce: Box::new(owl::Class(c_iri).into()),
                };
            }
        }

        // FIXME: is_class_level
        if self.is_class_level(&r_iri) {
            owl::ClassExpression::ObjectHasValue {
                o: owl::ObjectPropertyExpression::ObjectProperty(r_iri.into()),
                i: owl::NamedIndividual::from(c_iri),
            }
        } else {
            owl::ClassExpression::ObjectSomeValuesFrom {
                o: owl::ObjectPropertyExpression::ObjectProperty(r_iri.into()),
                ce: Box::new(owl::ClassExpression::Class(owl::Class(c_iri))),
            }
        }
    }
}
