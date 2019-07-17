use std::collections::HashSet;
use std::io::BufReader;
use std::path::Path;

use fastobo::ast as obo;
use fastobo::semantics::Identified;
use horned_owl::model as owl;

use super::into_owl::IntoOwlCtx;
use super::into_owl::Context;
use super::utils::hashset_take_arbitrary;

// ---------------------------------------------------------------------------

pub trait ImportProvider {
    fn import(&mut self, import: &obo::Import) -> Result<ImportData, String>;
}

// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct FoundryProvider {}

impl ImportProvider for FoundryProvider {
    fn import(&mut self, import: &obo::Import) -> Result<ImportData, String> {
        // use URL or use default OBO Foundry URL
        let url = match import {
            obo::Import::Url(url) => url.clone(),
            obo::Import::Abbreviated(id) => {
                let s = format!("http://purl.obolibrary.org/obo/{}.obo", id);
                obo::Url::parse(&s).expect("invalid import")
            }
        };

        // get the imported document
        let res = ureq::get(url.as_str()).redirects(10).call();
        let mut buf = BufReader::new(res.into_reader());

        // parse the OBO file if it is a correct OBO file.
        let mut data = match Path::new(url.path()).extension() {
            Some(x) if x == "obo" => {
                let mut doc = fastobo::ast::OboDoc::from_stream(&mut buf)
                    .expect("could not parse OBO document");
                doc.treat_xrefs();
                ImportData::from(doc)
            }
            Some(x) if x == "owl" => {
                unimplemented!("import OWL");
            }
            other => {
                panic!("unknown import extension: {:?}", other);
            }
        };

        // process all imports
        let mut imports = data.imports.clone();
        while let Some(i) = hashset_take_arbitrary(&mut imports) {
            // import the import in the document and add them to the `ImportData`.
            let import_data = self.import(&i)?;
            data.imports.extend(import_data.imports);
            data.annotation_properties.extend(import_data.annotation_properties);
        }

        Ok(data)
    }
}

// ---------------------------------------------------------------------------

/// The minimal data that
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportData {
    // Needed to check no class-level relationship is used in an
    // `intersection_of` clause (in theory...)
    // class_level_rel: HashSet<obo::RelationIdent>,

    /// The set of annotation properties declared in the document.
    ///
    /// Needed for for `rel(.., .., ..)` translation.
    pub annotation_properties: HashSet<owl::IRI>,

    /// The imports declared in the document.
    pub imports: HashSet<obo::Import>,
}

// ---------------------------------------------------------------------------

impl From<obo::OboDoc> for ImportData {
    fn from(doc: obo::OboDoc) -> Self {
        let mut annotation_properties = HashSet::new();
        let mut imports = HashSet::new();

        // create context to extract IRI
        let mut context = Context::from(&doc);

        // extract imports
        for clause in doc.header().iter() {
            if let obo::HeaderClause::Import(import) = clause {
                imports.insert(import.clone());
            }
        }

        // extract annotation properties
        for frame in doc.entities() {
            if let obo::EntityFrame::Typedef(typedef) = frame {
                for clause in typedef.clauses() {
                    if let obo::TypedefClause::IsMetadataTag(true) = clause.as_inner() {
                        let iri = frame.as_id().clone().into_owl(&mut context);
                        annotation_properties.insert(iri);
                    }
                }
            }
        }

        ImportData {
            annotation_properties,
            imports,
        }
    }
}
