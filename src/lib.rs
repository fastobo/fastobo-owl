#![warn(clippy::all)]

extern crate curie;
extern crate fastobo;
extern crate horned_owl;
#[macro_use]
extern crate lazy_static;

pub mod constants;

mod _context;
use _context::Context;

mod imports;
use imports::ImportData;

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

/// Create a [`curie::PrefixMapping`] instance with default prefixes declared.
///
/// The OBO Format 1.4 reference states that any OBO document translated into
/// OWL has the following prefixes declared implicitly: `xsd`, `owl`,
/// `oboInOwl`, `xml`, `rdf`, `dc` and `rdfs`.
///
/// [`curie::PrefixMapping`]: https://docs.rs/curie/0.0.8/curie/struct.PrefixMapping.html
pub fn obo_prefixes() -> curie::PrefixMapping {
    let mut prefixes = curie::PrefixMapping::default();
    prefixes.add_prefix("xsd", constants::uri::XSD).unwrap();
    prefixes.add_prefix("owl", constants::uri::OWL).unwrap();
    prefixes.add_prefix("obo", constants::uri::OBO).unwrap();
    prefixes.add_prefix("oboInOwl", constants::uri::OBO_IN_OWL).unwrap();
    prefixes.add_prefix("xml", constants::uri::XML).unwrap();
    prefixes.add_prefix("rdf", constants::uri::RDF).unwrap();
    prefixes.add_prefix("dc", constants::uri::DC).unwrap();
    prefixes.add_prefix("rdfs", constants::uri::RDFS).unwrap();
    prefixes
}
