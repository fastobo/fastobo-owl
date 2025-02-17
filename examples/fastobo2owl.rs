extern crate curie;
extern crate fastobo;
extern crate fastobo_owl;
extern crate horned_owl;

use std::rc::Rc;

use fastobo_owl::IntoOwl;
use fastobo_owl::IntoOwlPrefixes;
use horned_owl::model::AnnotatedComponent;
use horned_owl::ontology::component_mapped::ComponentMappedOntology;
use horned_owl::ontology::set::SetOntology;

fn main() {
    for arg in std::env::args().skip(1) {
        let path = std::path::PathBuf::from(arg);

        // Parse the document
        let obodoc = match fastobo::from_file(&path) {
            Ok(doc) => doc,
            Err(e) => panic!("{:?} could not be parsed:\n{}", path, e),
        };

        // Convert to OWL
        let prefixes = obodoc.prefixes();
        let owldoc: ComponentMappedOntology<String, Rc<AnnotatedComponent<String>>> =
            obodoc.into_owl::<SetOntology<String>>().unwrap().into();

        // Write it back
        let file = std::fs::File::create(path.with_extension("owl")).unwrap();
        let mut w = std::io::BufWriter::new(file);
        horned_owl::io::owx::writer::write(&mut w, &owldoc, Some(&prefixes)).unwrap();
        // horned_owl::io::rdf::writer::write(&mut w, &owldoc).unwrap();
    }
}
