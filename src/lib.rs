#![warn(clippy::all)]

extern crate curie;
extern crate fastobo;
extern crate horned_owl;
#[macro_use]
extern crate lazy_static;
extern crate ureq;

pub mod constants;
mod imports;
mod into_owl;
mod utils;

pub use imports::ImportData;
pub use into_owl::IntoOwl;

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
    prefixes
        .add_prefix("oboInOwl", constants::uri::OBO_IN_OWL)
        .unwrap();
    prefixes.add_prefix("xml", constants::uri::XML).unwrap();
    prefixes.add_prefix("rdf", constants::uri::RDF).unwrap();
    prefixes.add_prefix("dc", constants::uri::DC).unwrap();
    prefixes.add_prefix("rdfs", constants::uri::RDFS).unwrap();
    prefixes
}
