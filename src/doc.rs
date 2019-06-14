use std::collections::HashMap;

use fastobo::ast as obo;
use fastobo::semantics::Identified;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwl;
use super::IntoOwlCtx;
use crate::constants::uri;
use crate::imports::ImportData;

impl IntoOwlCtx for obo::OboDoc {
    type Owl = owl::Ontology;
    fn into_owl(mut self, ctx: &mut Context) -> Self::Owl {
        let mut ont = owl::Ontology::new();

        // TODO: declare the IRI and Version IRI for the ontology.
        // ont.id = owl::OntologyID {
        //     iri: Some(), // http://purl.obolibrary.org/obo/{ontology}.owl
        //     viri: Some(), // http://purl.obolibrary.org/obo/{ontology}/{data-version}/{ontology}.owl
        // }:

        // Convert the header frame: most frames end up as Ontology annotations,
        // but some of hem require extra axioms.
        let header = std::mem::replace(self.header_mut(), Default::default());
        for axiom in header.into_owl(ctx).into_iter() {
            ont.insert(axiom);
        }

        // Convert each entity to a set of OWL axioms that are then added to the ontology.
        let entities = std::mem::replace(self.entities_mut(), Default::default());
        for entity in entities.into_iter() {
            ctx.current_frame = entity.as_id().clone().into_owl(ctx);
            match entity {
                obo::EntityFrame::Term(frame) => {
                    for axiom in frame.into_owl(ctx) {
                        ont.insert(axiom);
                    }
                }
                // _ => unimplemented!(),
                _ => (),
            };
        }

        // Return the produced OWL ontology.
        ont
    }
}

impl IntoOwl for obo::OboDoc {
    type Owl = owl::Ontology;
    fn into_owl(mut self) -> Self::Owl {
        // Process the xref header macros.
        // Assigning the default namespace is not needed since we are only
        // processing the current document, so there should be no namespace
        // collision.
        self.treat_xrefs();

        // TODO: Process the imports
        // let data = ImportData::from(&self);

        // Create idspace mapping with implicit IDspaces.
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

        // Add the prefixes and IDspaces from the OBO header.
        let mut ontology = None;
        for clause in self.header() {
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

        // Create the conversion context.
        let build: horned_owl::model::Build = Default::default();
        let ontology_iri = obo::Url::parse(&format!("{}{}", uri::OBO, ontology.unwrap())).unwrap(); // FIXME
        let current_frame = build.iri(ontology_iri.clone().into_string());
        let class_level = Default::default(); // TODO: extract annotation properties
        let mut ctx = Context {
            build,
            idspaces,
            ontology_iri,
            current_frame,
            class_level,
        };

        // Return the converted document.
        <Self as IntoOwlCtx>::into_owl(self, &mut ctx)
    }
}
