use fastobo::ast as obo;
use fastobo::semantics::Identified;
use horned_owl::model::MutableOntology;
use horned_owl::ontology::set::SetOntology;

use super::Context;
use super::IntoOwl;
use super::IntoOwlCtx;

impl IntoOwlCtx for obo::OboDoc {
    type Owl = SetOntology;
    fn into_owl(mut self, ctx: &mut Context) -> Self::Owl {
        let mut ont = SetOntology::new();

        // TODO: declare the IRI and Version IRI for the ontology.
        // ont.id = owl::OntologyID {
        //     iri: Some(), // http://purl.obolibrary.org/obo/{ontology}.owl
        //     viri: Some(), // http://purl.obolibrary.org/obo/{ontology}/{data-version}/{ontology}.owl
        // }:

        // Convert the header frame: most frames end up as Ontology annotations,
        // but some of hem require extra axioms.
        let header = std::mem::take(self.header_mut());
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
                obo::EntityFrame::Typedef(frame) => {
                    for axiom in frame.into_owl(ctx) {
                        ont.insert(axiom);
                    }
                }
                _ => (), // NB: individuals are ignored
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
                mapping.add_prefix(prefix.as_str(), url.as_str()).ok();
            }
        }
        mapping
    }

    fn into_owl(mut self) -> SetOntology {
        // Assign default namespaces to entities missing one.
        self.assign_namespaces().ok(); // ignore errors
        // Process the xref header macros.
        self.treat_xrefs();

        // Extract conversion context from the document.
        let mut ctx = Context::from(&self);

        // // TODO: Extract the data needed for conversion from the imports.
        // let mut provider = FoundryProvider::default();
        // for clause in self.header() {
        //     if let obo::HeaderClause::Import(i) = clause {
        //         let data = provider.import(i).unwrap(); // FIXME: chain error
        //         ctx.class_level.extend(data.annotation_properties);
        //     }
        // }

        // Return the converted document.
        <Self as IntoOwlCtx>::into_owl(self, &mut ctx)
    }
}
