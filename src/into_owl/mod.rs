mod date;
mod doc;
mod header;
mod id;
mod pv;
mod qualifier;
mod strings;
mod syn;
mod term;
mod typedef;
mod xref;

use std::collections::HashMap;
use std::collections::HashSet;

use fastobo::ast as obo;
use horned_owl::model as owl;

use crate::constants::uri;

// ---------------------------------------------------------------------------

/// The internal trait for data conversion;
///
/// This is not exposed because `ctx` can be mostly inferred from the source
/// OBO ontology, therefore a public trait shall be made available only for
/// the `OboDoc` struct, with less arguments to provide.
pub trait IntoOwlCtx {
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
    ///
    /// See also: [`horned_owl::io::writer::write`](https://docs.rs/horned-owl/latest/horned_owl/io/writer/fn.write.html).
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
    /// The `horned_owl::model::Build` to create reference counted IRI.
    pub build: owl::Build,

    /// A mapping of the declared OBO ID spaces to their respective URL bases.
    pub idspaces: HashMap<obo::IdentPrefix, obo::Url>,

    /// The IRI of the ontology currently being processed.
    pub ontology_iri: obo::Url,

    /// The IRI of the frame currently being processed.
    pub current_frame: owl::IRI,

    /// A flag to indicate the current frame is an annotation property.
    pub in_annotation: bool,

    /// A mapping
    pub shorthands: HashMap<obo::UnprefixedIdent, obo::Ident>,

    /// A set of IRI which refer to class level annotation relationships.
    ///
    /// This is likely to require processing imports beforehand.
    pub class_level: HashSet<owl::IRI>,
}

impl Context {
    pub fn find_shorthand(frame: &obo::TypedefFrame) -> Option<obo::Ident> {
        None
    }

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
                    ope: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(r_iri)),
                    bce: Box::new(owl::ClassExpression::ObjectComplementOf(
                        Box::new(owl::Class(c_iri).into()),
                    )),
                };
            } else {
                return owl::ClassExpression::ObjectExactCardinality {
                    n,
                    ope: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(r_iri)),
                    bce: Box::new(owl::Class(c_iri).into()),
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
                    ope: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(r_iri)),
                    bce: Box::new(owl::ClassExpression::ObjectComplementOf(
                        Box::new(owl::Class(c_iri).into()),
                    )),
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
                    .expect("invalid value for `max_cardinality`");
                return owl::ClassExpression::ObjectIntersectionOf(
                    vec![
                        owl::ClassExpression::ObjectMinCardinality {
                            n: na,
                            ope: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(
                                r_iri.clone(),
                            )),
                            bce: Box::new(owl::Class(c_iri.clone()).into()),
                        },
                        owl::ClassExpression::ObjectMaxCardinality {
                            n: nb,
                            ope: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(
                                r_iri.clone(),
                            )),
                            bce: Box::new(owl::Class(c_iri.clone()).into()),
                        },
                    ],
                );
            } else {
                return owl::ClassExpression::ObjectMinCardinality {
                    n: na,
                    ope: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(r_iri)),
                    bce: Box::new(owl::Class(c_iri).into()),
                };
            }
        }

        if let Some(q) = qualifiers.iter().find(|q| q.key() == &*MAX_CARDINALITY) {
            return owl::ClassExpression::ObjectMaxCardinality {
                n: q.value()
                    .parse()
                    .expect("invalid value for `maxCardinality`"),
                ope: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(r_iri)),
                bce: Box::new(owl::ClassExpression::ObjectComplementOf(
                    Box::new(owl::Class(c_iri).into()),
                )),
            };
        }

        if let Some(q_only) = qualifiers.iter().find(|q| q.key() == &*ALL_ONLY) {
            if let Some(q_some) = qualifiers.iter().find(|q| q.key() == &*ALL_SOME) {
                return owl::ClassExpression::ObjectIntersectionOf(
                    vec![
                        owl::ClassExpression::ObjectSomeValuesFrom {
                            ope: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(
                                r_iri.clone(),
                            )),
                            bce: Box::new(owl::Class(c_iri.clone()).into()),
                        },
                        owl::ClassExpression::ObjectAllValuesFrom {
                            ope: owl::ObjectPropertyExpression::ObjectProperty(owl::ObjectProperty(
                                r_iri,
                            )),
                            bce: Box::new(owl::Class(c_iri).into()),
                        },
                    ],
                );
            } else {
                return owl::ClassExpression::ObjectAllValuesFrom {
                    ope: owl::ObjectPropertyExpression::ObjectProperty(r_iri.into()),
                    bce: Box::new(owl::Class(c_iri).into()),
                };
            }
        }

        // FIXME: is_class_level
        if self.is_class_level(&r_iri) {
            owl::ClassExpression::ObjectHasValue {
                ope: owl::ObjectPropertyExpression::ObjectProperty(r_iri.into()),
                i: owl::NamedIndividual::from(c_iri),
            }
        } else {
            owl::ClassExpression::ObjectSomeValuesFrom {
                ope: owl::ObjectPropertyExpression::ObjectProperty(r_iri.into()),
                bce: Box::new(owl::ClassExpression::Class(owl::Class(c_iri))),
            }
        }
    }
}

impl From<&obo::OboDoc> for Context {
    fn from(doc: &obo::OboDoc) -> Self {
        // Add the ID spaces declared implicitly in the document.
        let mut idspaces = HashMap::new();
        idspaces.insert(
            obo::IdentPrefix::new("BFO"),
            obo::Url::parse(&format!("{}BFO_", uri::OBO,)).unwrap(),
        );
        idspaces.insert(
            obo::IdentPrefix::new("RO"),
            obo::Url::parse(&format!("{}RO", uri::OBO,)).unwrap(),
        );
        idspaces.insert(
            obo::IdentPrefix::new("xsd"),
            obo::Url::parse(uri::XSD).unwrap(),
        );

        // Add the prefixes and ID spaces from the OBO header.
        let mut ontology = None;
        for clause in doc.header() {
            match clause {
                obo::HeaderClause::Idspace(prefix, url, _) => {
                    idspaces.insert(prefix.clone(), url.clone());
                }
                obo::HeaderClause::Ontology(id) => {
                    ontology = Some(id.to_string());
                }
                _ => (),
            }
        }

        // Add the shorthands from the OBO typdef
        let mut shorthands = HashMap::new();
        for frame in doc.entities() {
            if let obo::EntityFrame::Typedef(typedef) = frame {
                let id = typedef.id().as_ref().as_ref();
                if let obo::Ident::Unprefixed(unprefixed) = id {
                    if let Some(short) = Context::find_shorthand(typedef) {
                        shorthands.insert(id.clone(), short.clone());
                    }
                }
            }
        }

        // Create the conversion context.
        let build: horned_owl::model::Build = Default::default();
        let ontology_iri = obo::Url::parse(&format!("{}{}", uri::OBO, ontology.unwrap())).unwrap(); // FIXME
        let current_frame = build.iri(ontology_iri.clone().into_string());
        let class_level = Default::default(); // TODO: extract annotation properties
        let shorthands = Default::default(); // TODO: extract shorthands

        Context {
            build,
            idspaces,
            ontology_iri,
            current_frame,
            class_level,
            shorthands,
            in_annotation: false,
        }
    }
}
