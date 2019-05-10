extern crate curie;
extern crate fastobo;
extern crate horned_owl;

pub mod constants;

mod doc;
mod header;
mod id;
mod term;

use std::collections::HashMap;

use fastobo::ast as obo;
use horned_owl::model as owl;




/// The internal trait for data conversion;
///
/// This is not exposed because `ctx` can be mostly inferred from the source
/// OBO ontology, therefore a public trait shall be made available only for
/// the `OboDoc` struct, with less arguments to provide.s
trait IntoOwlCtx {
    type Owl;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl;
}

/// The public conversion trait for structs that can be converted to OWL.
pub trait IntoOwl {
    type Owl;
    fn into_owl(self) -> Self::Owl;
}

/// An opaque structure to pass context arguments required for OWL conversion.
struct Context {
    build: owl::Build,
    prefixes: curie::PrefixMapping,
    idspaces: HashMap<obo::IdentPrefix, obo::Url>,
    ontology_iri: obo::Url,
    current_frame: owl::IRI,
}

/// An entity produced by a certain clause.
///
/// Some OBO clauses are translated as annotations, some other are translated
/// into axioms, and some have no translation.
enum OwlEntity {
    Annotation(owl::Annotation),
    Axiom(owl::Axiom),
    None
}

impl From<owl::Annotation> for OwlEntity {
    fn from(a: owl::Annotation) -> Self {
        OwlEntity::Annotation(a)
    }
}

impl From<owl::Axiom> for OwlEntity {
    fn from(a: owl::Axiom) -> Self {
        OwlEntity::Axiom(a)
    }
}
