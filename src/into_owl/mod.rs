mod date;
mod def;
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
use std::convert::TryFrom;
use std::ops::Deref;

use fastobo::ast as obo;
use fastobo::error::CardinalityError;
use horned_owl::model as owl;
use horned_owl::model::MutableOntology;
use horned_owl::model::Ontology;

use crate::constants::uri;
use crate::error::Error;

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
    fn into_owl<O>(self) -> Result<O, Error>
    where
        O: Default + Ontology + MutableOntology;
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
#[derive(Debug)]
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

    /// A set of IRI which refer to class level relationships.
    ///
    /// This is likely to require processing imports beforehand.
    pub class_level: HashSet<owl::IRI>,

    /// A set or IRI which refer to metadata tag typedef.
    ///
    /// Properties that are marked as metadata tags are used to record object
    /// metadata and are translated to annotation properties.
    pub metadata_tag: HashSet<owl::IRI>,
}

impl Context {
    pub fn from_obodoc(doc: &obo::OboDoc) -> Result<Self, Error> {
        // Add the ID spaces declared implicitly in the document.
        let mut idspaces = HashMap::new();
        idspaces.insert(
            obo::IdentPrefix::new("BFO"),
            obo::Url::new(format!("{}BFO_", uri::OBO,)).unwrap(),
        );
        idspaces.insert(
            obo::IdentPrefix::new("RO"),
            obo::Url::new(format!("{}RO", uri::OBO,)).unwrap(),
        );
        idspaces.insert(
            obo::IdentPrefix::new("xsd"),
            obo::Url::new(uri::XSD).unwrap(),
        );

        // Add the prefixes and ID spaces from the OBO header.
        let mut ontology = Err(Error::Cardinality(CardinalityError::missing("ontology")));
        for clause in doc.header() {
            match clause {
                obo::HeaderClause::Idspace(prefix, url, _) => {
                    idspaces.insert(prefix.deref().clone(), url.deref().clone());
                }
                obo::HeaderClause::Ontology(id) if ontology.is_err() => {
                    ontology = Ok(id.to_string());
                }
                obo::HeaderClause::Ontology(_) if ontology.is_ok() => {
                    return Err(Error::Cardinality(CardinalityError::duplicate("ontology")));
                }
                _ => (),
            }
        }

        // Add the shorthands from the OBO typdef
        let mut shorthands = HashMap::new();
        for frame in doc
            .entities()
            .iter()
            .flat_map(obo::EntityFrame::as_typedef_frame)
        {
            let id = frame.id().as_ref().as_ref();
            if let obo::Ident::Unprefixed(unprefixed) = id {
                if let Some(short) = Context::find_shorthand(frame) {
                    shorthands.insert(unprefixed.deref().clone(), short.clone());
                }
            }
        }

        // Create the conversion context.
        let build: horned_owl::model::Build = Default::default();
        let ontology_iri = obo::Url::new(format!("{}{}", uri::OBO, ontology?))?;
        let current_frame = build.iri(ontology_iri.as_str().to_string());
        let mut ctx = Context {
            build,
            idspaces,
            ontology_iri,
            current_frame,
            shorthands,
            metadata_tag: Default::default(),
            class_level: Default::default(),
            in_annotation: false,
        };

        // Retrieve class-level relationships and annotation properties
        //
        // NB: this is done after the context is created because we need to
        //     perform OBO ID to IRI conversion for the typedefs, which
        //     already requires a context (in case the typedef has a prefixed
        //     identifier).
        for frame in doc
            .entities()
            .iter()
            .flat_map(obo::EntityFrame::as_typedef_frame)
        {
            let is_metadata_tag = frame.iter().any(|line| match line.as_inner() {
                obo::TypedefClause::IsMetadataTag(true) => true,
                _ => false,
            });
            let is_class_level = frame.iter().any(|line| match line.as_inner() {
                obo::TypedefClause::IsClassLevel(true) => true,
                _ => false,
            });
            if is_metadata_tag || is_class_level {
                let iri = frame.id().as_ref().clone().into_owl(&mut ctx);
                if is_class_level {
                    ctx.class_level.insert(iri.clone());
                }
                if is_metadata_tag {
                    ctx.metadata_tag.insert(iri.clone());
                }
            }
        }

        Ok(ctx)
    }

    pub fn find_shorthand(frame: &obo::TypedefFrame) -> Option<&obo::Ident> {
        if let obo::Ident::Unprefixed(_) = frame.id().as_inner().as_ref() {
            // FIXME: right now this takes the first xref of a typedef,
            //        assuming there is only one, but priority rules from
            //        the OBO 1.4 specs should be implemented.
            frame.iter().find_map(|c| match c.as_inner() {
                obo::TypedefClause::Xref(x) => match x.id() {
                    i @ obo::Ident::Prefixed(_) => Some(i),
                    i @ obo::Ident::Url(_) => Some(i),
                    _ => None,
                },
                _ => None,
            })
        } else {
            None
        }
    }

    pub fn is_class_level(&mut self, rid: &owl::IRI) -> bool {
        self.class_level.contains(&rid)
    }

    pub fn is_metadata_tag(&mut self, rid: &owl::IRI) -> bool {
        self.metadata_tag.contains(&rid)
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
                    bce: Box::new(owl::ClassExpression::ObjectComplementOf(Box::new(
                        owl::Class(c_iri).into(),
                    ))),
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
                    bce: Box::new(owl::ClassExpression::ObjectComplementOf(Box::new(
                        owl::Class(c_iri).into(),
                    ))),
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
                return owl::ClassExpression::ObjectIntersectionOf(vec![
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
                ]);
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
                bce: Box::new(owl::ClassExpression::ObjectComplementOf(Box::new(
                    owl::Class(c_iri).into(),
                ))),
            };
        }

        if qualifiers.iter().any(|q| q.key() == &*ALL_ONLY) {
            if qualifiers.iter().any(|q| q.key() == &*ALL_SOME) {
                return owl::ClassExpression::ObjectIntersectionOf(vec![
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
                ]);
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
                i: owl::Individual::Named(owl::NamedIndividual::from(c_iri)),
            }
        } else {
            owl::ClassExpression::ObjectSomeValuesFrom {
                ope: owl::ObjectPropertyExpression::ObjectProperty(r_iri.into()),
                bce: Box::new(owl::ClassExpression::Class(owl::Class(c_iri))),
            }
        }
    }
}

impl TryFrom<&obo::OboDoc> for Context {
    type Error = Error;
    fn try_from(doc: &obo::OboDoc) -> Result<Self, Error> {
        Self::from_obodoc(doc)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn missing_ontology_clause() {
        let doc = obo::OboDoc::new();
        let res = Context::from_obodoc(&doc);
        assert!(matches!(
            res,
            Err(Error::Cardinality(CardinalityError::MissingClause { .. }))
        ));
    }

    #[test]
    fn duplicate_ontology_clause() {
        let mut doc = obo::OboDoc::new();
        doc.header_mut().push(obo::HeaderClause::Ontology(Box::new(
            obo::UnquotedString::new("test1"),
        )));
        doc.header_mut().push(obo::HeaderClause::Ontology(Box::new(
            obo::UnquotedString::new("test2"),
        )));

        let res = Context::from_obodoc(&doc);
        assert!(matches!(
            res,
            Err(Error::Cardinality(
                CardinalityError::DuplicateClauses { .. }
            ))
        ));
    }
}
