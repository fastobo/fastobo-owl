use fastobo::ast as obo;
use fastobo::semantics::Identified;
use horned_owl::model::AnnotatedComponent;
use horned_owl::model::Component;
use horned_owl::model::ForIRI;
use horned_owl::model::Import;
use horned_owl::model::MutableOntology;
use horned_owl::model::OntologyID;

use super::Context;
use super::IntoOwl;
use super::IntoOwlCtx;
use super::IntoOwlPrefixes;
use crate::constants::uri;
use crate::error::Error;

impl IntoOwlPrefixes for obo::OboDoc {
    fn prefixes(&self) -> curie::PrefixMapping {
        let mut mapping = crate::obo_prefixes();
        for clause in self.header() {
            if let obo::HeaderClause::Idspace(prefix, url, _) = clause {
                mapping.add_prefix(prefix.as_str(), url.as_str()).ok();
            }
        }
        mapping
    }
}

impl<A: ForIRI> IntoOwl<A> for obo::OboDoc {
    fn into_owl<O>(mut self) -> Result<O, Error>
    where
        O: Default + MutableOntology<A>,
    {
        // Assign default namespaces to entities missing one.
        self.assign_namespaces()?; // ignore errors

        // Process the xref header macros.
        self.treat_xrefs();

        // Extract conversion context from the document.
        let mut ctx = Context::from_obodoc(&self)?;

        // Create the output ontology
        let mut ont = O::default();

        // declare the IRI and Version IRI for the ontology.
        let id = if let Ok(name) = self.header().ontology() {
            OntologyID {
                iri: Some(ctx.build.iri(format!("{}{}.owl", uri::OBO, name))),
                viri: self
                    .header()
                    .data_version()
                    .map(|dv| {
                        ctx.build
                            .iri(format!("{}{}/{}/{}.owl", uri::OBO, name, dv, name))
                    })
                    .ok(),
            }
        } else {
            OntologyID::default()
        };
        ont.insert(AnnotatedComponent::from(Component::OntologyID(id)));

        // Convert the header frame: most frames end up as Ontology annotations,
        // but some of them require extra axioms.
        let header = std::mem::take(self.header_mut());
        for axiom in header.into_owl(&mut ctx).into_iter() {
            ont.insert(axiom);
        }

        // force import of the oboInOwl ontology
        ont.insert(Component::Import(Import(
            ctx.build
                .iri("http://www.geneontology.org/formats/oboInOwl"),
        )));

        // Convert each entity to a set of OWL axioms that are then added to the ontology.
        let entities = std::mem::replace(self.entities_mut(), Default::default());
        for entity in entities.into_iter() {
            ctx.current_frame = entity.as_id().clone().into_owl(&mut ctx);
            match entity {
                obo::EntityFrame::Term(frame) => {
                    for axiom in frame.into_owl(&mut ctx) {
                        ont.insert(axiom);
                    }
                }
                obo::EntityFrame::Typedef(frame) => {
                    for axiom in frame.into_owl(&mut ctx) {
                        ont.insert(axiom);
                    }
                }
                _ => (), // NB: individuals are ignored
            };
        }

        // Return the produced OWL ontology.
        Ok(ont)
    }
}
