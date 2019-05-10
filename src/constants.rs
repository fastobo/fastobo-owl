//! Constant datatype IRIs and base URIs used literally during translation.

// --- Macros to allow const concatenation in submodules ---------------------

macro_rules! uris {
    ($($name:ident => $uri:literal,)*) => (
        $(macro_rules! $name {() => ($uri)})*
    );
}

uris! {
    dc => "http://purl.org/dc/elements/1.1/",
    obo => "http://purl.obolibrary.org/obo/",
    oboInOwl => "http://www.geneontology.org/formats/oboInOwl#",
    owl => "http://www.w3.org/2002/07/owl#",
    rdf => "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
    rdfs => "http://www.w3.org/2000/01/rdf-schema#",
    xml => "http://www.w3.org/XML/1998/namespace",
    xsd => "http://www.w3.org/2001/XMLSchema#",
}

// --- Base URIs -------------------------------------------------------------
pub mod uri {
    pub const DC: &'static str = dc!();
    pub const OBO: &'static str = obo!();
    pub const OBO_IN_OWL: &'static str = oboInOwl!();
    pub const OWL: &'static str = owl!();
    pub const RDF: &'static str = rdf!();
    pub const RDFS: &'static str = rdfs!();
    pub const XML: &'static str = xml!();
    pub const XSD: &'static str = xsd!();
}

// --- Datatype URIs ---------------------------------------------------------

/// Datatypes used in OBO to OWL translation.
pub mod datatype {
    /// XML Schema datatypes.
    pub mod xsd {
        pub const STRING: &'static str = concat!(xsd!(), "string");
        pub const BOOLEAN: &'static str = concat!(xsd!(), "boolean");
        pub const DATETIME: &'static str = concat!(xsd!(), "dateTime");
    }
}


// --- Annotation Property URIs ----------------------------------------------

/// Annotation properties used to expose OBO semantics in OWL.
pub mod property {
    /// OBO in OWL common annotation properties.
    pub mod obo_in_owl {
        pub const AUTO_GENERATED_BY: &'static str = concat!(oboInOwl!(), "autoGeneratedBy");
        pub const CONSIDER: &'static str = concat!(oboInOwl!(), "consider");
        pub const HAS_DATE: &'static str = concat!(oboInOwl!(), "hasDate");
        pub const HAS_DBXREF: &'static str = concat!(oboInOwl!(), "hasDbXref");
        pub const HAS_DEFAULT_NAMESPACE: &'static str = concat!(oboInOwl!(), "hasDefaultNamespace");
        pub const HAS_OBO_FORMAT_VERSION: &'static str = concat!(oboInOwl!(), "hasOBOFormatVersion");
        pub const ID: &'static str = concat!(oboInOwl!(), "id");
        pub const NAMESPACE_ID_RULE: &'static str = concat!(oboInOwl!(), "NamespaceIdRule");
        pub const SAVED_BY: &'static str = concat!(oboInOwl!(), "savedBy");
        pub const SUBSET_PROPERTY: &'static str = concat!(oboInOwl!(), "SubsetProperty");
        pub const SYNONYM_TYPE_PROPERTY: &'static str = concat!(oboInOwl!(), "SynonymTypeProperty");
    }

    /// RDF Schema annotation properties.
    pub mod rdfs {
        pub const COMMENT: &'static str = concat!(rdfs!(), "comment");
        pub const SUB_PROPERTY_OF: &'static str = concat!(rdfs!(), "subPropertyOf");
    }

    /// Dublin core annotation properties.
    pub mod dc {
        pub const CREATOR: &'static str = concat!(dc!(), "creator");
        pub const DATE: &'static str = concat!(dc!(), "date");
    }
}
