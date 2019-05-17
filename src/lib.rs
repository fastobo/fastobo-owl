#![warn(clippy::all)]

extern crate curie;
extern crate fastobo;
extern crate horned_owl;

pub mod constants;

mod date;
mod doc;
mod header;
mod id;
mod pv;
mod strings;
mod syn;
mod term;
mod xref;

use std::collections::HashMap;
use std::collections::HashSet;

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
    ///
    build: owl::Build,

    // prefixes: curie::PrefixMapping,
    idspaces: HashMap<obo::IdentPrefix, obo::Url>,

    ontology_iri: obo::Url,

    current_frame: owl::IRI,

    /// A set of IRI which refer to class level annotation relationships.
    ///
    /// This is likely to require processing imports beforehand.
    class_level: HashSet<owl::IRI>,
}
