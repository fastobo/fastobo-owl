use std::collections::HashSet;

use fastobo::ast as obo;
use horned_owl::model as owl;



// pub trait Provider {
//     fn import()
// }
//
//
// pub struct FoundryProvider {
//
// }




pub struct ImportData {
    // Needed to check no class-level relationship is used in an
    // `intersection_of` clause (in theory...)
    // class_level_rel: HashSet<obo::RelationIdent>,

    // Needed for for rel(.., .., ..) translation
    pub annotation_properties: HashSet<owl::IRI>,
}

// impl From<&obo::OboDoc> for ImportData {
//     fn from(doc: &obo::OboDoc) -> Self {
//
//         let clause = obo::Eol::new().and_inner(obo::TypedefClause::IsMetadataTag(true));
//
//         Self {
//             annotation_properties: doc
//                 .entities()
//                 .iter()
//                 .filter_map(obo::EntityFrame::as_typedef_frame)
//                 .filter(|f| f.contains(&clause))
//                 .map(|f| f.as_ident())
//                 .cloned()
//                 .collect(),
//         }
//     }
// }


//
// fn process_obo(doc: &obo::OboDoc) -> ImportData {
//     // Extract the imports
//     let _imports: Vec<obo::Url> = doc
//         .header()
//         .iter()
//         .filter_map(|clause| {
//             if let obo::HeaderClause::Import(i) = clause {
//                 Some(i.clone().into_url())
//             } else {
//                 None
//             }
//         })
//         .collect();
//
//     //
//     ForeignRequirements {
//             metadata_tags: doc
//             .entities()
//             .iter()
//             .filter_map(obo::EntityFrame::as_typedef_frame)
//             .filter(|f| f.contains(&obo::TypedefClause::IsMetadataTag(true)))
//             .map(|f| f.as_ident())
//             .collect(),
//     }
// }
