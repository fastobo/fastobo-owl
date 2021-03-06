extern crate fastobo;
extern crate fastobo_owl;
extern crate lazy_static;
extern crate pretty_assertions;

use std::path::PathBuf;

use pretty_assertions::assert_eq;

use fastobo::ast::OboDoc;
use fastobo_owl::IntoOwl;

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
            let actual = obo_doc.into_owl();

            // horned_owl::io::writer::write(&mut std::io::stdout(), &actual, Some(&PREFIXES));

            // Read the expected OWL
            let (expected, _prefixes) = horned_owl::io::reader::read(&mut std::io::BufReader::new(
                std::fs::File::open(&output_path).expect("could not open output file"),
            ))
            .expect("could not parse output file");

            // Test equality
            assert_eq!(actual, expected);
        }
    };
}

converttest!(def_xref);
converttest!(equivalent_to);
converttest!(header);
converttest!(intersection_of);
converttest!(is_a);
converttest!(name);
converttest!(qualifier);
converttest!(union_of);
converttest!(synonym);
