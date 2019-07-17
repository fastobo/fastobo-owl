use std::collections::HashMap;

use fastobo::ast as obo;
use fastobo::semantics::Identified;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwl;
use super::IntoOwlCtx;
use crate::constants::uri;
use crate::imports::ImportData;
use crate::imports::ImportProvider;
use crate::imports::FoundryProvider;

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
    fn prefixes(&self) -> curie::PrefixMapping {
        let mut mapping = crate::obo_prefixes();
        for clause in self.header() {
            if let obo::HeaderClause::Idspace(prefix, url, _) = clause {
                mapping.add_prefix(prefix.as_str(), url.as_str());
            }
        }
        mapping
    }

    fn into_owl(mut self) -> owl::Ontology {
        // Process the xref header macros.
        // Assigning the default namespace is not needed since we are only
        // processing the current document, so there should be no namespace
        // collision.
        self.treat_xrefs();

        // Extract conversion context from the document.
        let mut ctx = Context::from(&self);

        // Extract the data needed for conversion from the imports.
        let mut provider = FoundryProvider::default();
        for clause in self.header() {
            if let obo::HeaderClause::Import(i) = clause {
                let data = provider.import(i).unwrap(); // FIXME: chain error
                ctx.class_level.extend(data.annotation_properties);
            }
        }

        // Return the converted document.
        <Self as IntoOwlCtx>::into_owl(self, &mut ctx)
    }
}
