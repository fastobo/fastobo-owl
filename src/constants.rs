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
    pub const DC: &str = dc!();
    pub const OBO: &str = obo!();
    pub const OBO_IN_OWL: &str = oboInOwl!();
    pub const OWL: &str = owl!();
    pub const RDF: &str = rdf!();
    pub const RDFS: &str = rdfs!();
    pub const XML: &str = xml!();
    pub const XSD: &str = xsd!();
}

// --- Datatype URIs ---------------------------------------------------------

/// Datatypes used in OBO to OWL translation.
pub mod datatype {
    /// XML Schema datatypes.
    pub mod xsd {
        pub const STRING: &str = concat!(xsd!(), "string");
        pub const BOOLEAN: &str = concat!(xsd!(), "boolean");
        pub const DATETIME: &str = concat!(xsd!(), "dateTime");
    }
}

// --- Annotation Property URIs ----------------------------------------------

/// Annotation properties used to expose OBO semantics in OWL.
pub mod property {
    /// OBO in OWL common annotation properties.
    pub mod obo_in_owl {
        pub const AUTO_GENERATED_BY: &str = concat!(oboInOwl!(), "autoGeneratedBy");
        pub const CONSIDER: &str = concat!(oboInOwl!(), "consider");
        pub const HAS_ALTERNATIVE_ID: &str = concat!(oboInOwl!(), "hasAlternativeId");
        pub const HAS_DATE: &str = concat!(oboInOwl!(), "hasDate");
        pub const HAS_DBXREF: &str = concat!(oboInOwl!(), "hasDbXref");
        pub const HAS_DEFAULT_NAMESPACE: &str = concat!(oboInOwl!(), "hasDefaultNamespace");
        pub const HAS_OBO_FORMAT_VERSION: &str = concat!(oboInOwl!(), "hasOBOFormatVersion");
        pub const HAS_OBO_NAMESPACE: &str = concat!(oboInOwl!(), "hasOBONamespace");
        pub const ID: &str = concat!(oboInOwl!(), "id");
        pub const IN_SUBSET: &str = concat!(oboInOwl!(), "inSubset");
        pub const NAMESPACE_ID_RULE: &str = concat!(oboInOwl!(), "NamespaceIdRule");
        pub const SAVED_BY: &str = concat!(oboInOwl!(), "savedBy");
        pub const SUBSET_PROPERTY: &str = concat!(oboInOwl!(), "SubsetProperty");
        pub const SYNONYM_TYPE_PROPERTY: &str = concat!(oboInOwl!(), "SynonymTypeProperty");

        pub const HAS_BROAD_SYNONYM: &str = concat!(oboInOwl!(), "hasBroadSynonym");
        pub const HAS_EXACT_SYNONYM: &str = concat!(oboInOwl!(), "hasExactSynonym");
        pub const HAS_NARROW_SYNONYM: &str = concat!(oboInOwl!(), "hasNarrowSynonym");
        pub const HAS_RELATED_SYNONYM: &str = concat!(oboInOwl!(), "hasRelatedSynonym");
        pub const HAS_SYNONYM_TYPE: &str = concat!(oboInOwl!(), "hasSynonymType");
    }

    /// OWL2 annotation properties.
    pub mod owl {
        pub const DEPRECATED: &str = concat!(owl!(), "deprecated");
    }

    /// RDF Schema annotation properties.
    pub mod rdfs {
        pub const LABEL: &str = concat!(rdfs!(), "label");
        pub const COMMENT: &str = concat!(rdfs!(), "comment");
        pub const SUB_PROPERTY_OF: &str = concat!(rdfs!(), "subPropertyOf");
    }

    /// Dublin core annotation properties.
    pub mod dc {
        pub const CREATOR: &str = concat!(dc!(), "creator");
        pub const DATE: &str = concat!(dc!(), "date");
    }

    /// Information artifact ontology annotation properties.
    pub mod iao {
        pub const REPLACED_BY: &str = concat!(obo!(), "IAO_0100001");
        pub const DEFINITION: &str = concat!(obo!(), "IAO_0000115");
    }
}
