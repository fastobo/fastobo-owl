extern crate fastobo;
extern crate fastobo_owl;
extern crate lazy_static;
extern crate pretty_assertions;

use std::path::PathBuf;

use fastobo_owl::IntoOwl;
use horned_owl::model::Build;
use horned_owl::ontology::set::SetOntology;
use pretty_assertions::assert_eq;

macro_rules! converttest {
    ($name:ident) => {
        #[test]
        fn $name() {
            let dir = {
                let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                p.push("tests");
                p.push("data");
                p.push("into_owl");
                p
            };

            let input_path = dir.join(format!("{}.input.obo", stringify!($name)));
            let output_path = dir.join(format!("{}.output.owl", stringify!($name)));

            // Parse the OBO doc and convert it to OWL.
            let obo_doc = fastobo::from_file(&input_path).expect("could not parse input file");
            let actual = obo_doc
                .into_owl::<SetOntology<String>>()
                .expect("could not convert ontology to OWL");

            // Read the expected OWL
            let mut reader = std::fs::File::open(&output_path)
                .map(std::io::BufReader::new)
                .expect("could not open output file");
            let (expected, _prefixes) =
                horned_owl::io::owx::reader::read_with_build(&mut reader, &Build::new())
                    .expect("could not parse output file");

            // reorder
            let mut exp: Vec<_> = expected.iter().collect();
            exp.sort();
            let mut act: Vec<_> = actual.iter().collect();
            act.sort();

            // println!("act={:#?}", act);

            assert_eq!(act, exp);
        }
    };
}

converttest!(def_xref);
converttest!(equivalent_to);
converttest!(header);
converttest!(intersection_of);
converttest!(is_a);
converttest!(name);
converttest!(property_value);
converttest!(qualifier);
converttest!(subsetdef);
converttest!(synonym);
converttest!(union_of);
converttest!(typedef_shorthand);
converttest!(version_iri);
