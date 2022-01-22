use fastobo::ast as obo;
use fastobo::semantics::Identified;
use horned_owl::model::MutableOntology;
use horned_owl::model::Ontology;

use super::Context;
use super::IntoOwl;
use super::IntoOwlCtx;
use crate::constants::uri;
use crate::error::Error;

impl IntoOwl for obo::OboDoc {
    fn prefixes(&self) -> curie::PrefixMapping {
        let mut mapping = crate::obo_prefixes();
        for clause in self.header() {
            if let obo::HeaderClause::Idspace(prefix, url, _) = clause {
                mapping.add_prefix(prefix.as_str(), url.as_str()).ok();
            }
        }
        mapping
    }

    fn into_owl<O>(mut self) -> Result<O, Error>
    where
        O: Default + Ontology + MutableOntology,
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
        if let Some(name) = self.header().iter().find_map(|c| match c {
            obo::HeaderClause::Ontology(name) => Some(name),
            _ => None,
        }) {
            // persistent URL
            let url = format!("{}{}.owl", uri::OBO, name);
            ont.mut_id().iri = Some(ctx.build.iri(url));
            // version-specific URL
            if let Ok(dv) = self.header().data_version() {
                let url = format!("{}{}/{}/{}.owl", uri::OBO, name, dv, name);
                ont.mut_id().viri = Some(ctx.build.iri(url));
            }
        }

        // Convert the header frame: most frames end up as Ontology annotations,
        // but some of them require extra axioms.
        let header = std::mem::take(self.header_mut());
        for axiom in header.into_owl(&mut ctx).into_iter() {
            ont.insert(axiom);
        }

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
