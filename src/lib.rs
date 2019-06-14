#![warn(clippy::all)]

extern crate curie;
extern crate fastobo;
extern crate horned_owl;
#[macro_use]
extern crate lazy_static;

pub mod constants;

mod _context;
use _context::Context;

mod imports;
use imports::ImportData;

mod date;
mod doc;
mod header;
mod id;
mod pv;
mod qualifier;
mod strings;
mod syn;
mod term;
mod xref;

use std::collections::HashMap;
use std::collections::HashSet;

use fastobo::ast as obo;
use horned_owl::model as owl;

// ---------------------------------------------------------------------------

/// The internal trait for data conversion;
///
/// This is not exposed because `ctx` can be mostly inferred from the source
/// OBO ontology, therefore a public trait shall be made available only for
/// the `OboDoc` struct, with less arguments to provide.
trait IntoOwlCtx {
    type Owl;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl;
}

/// The public conversion trait for structs that can be converted to OWL.
pub trait IntoOwl {
    type Owl;
    fn into_owl(self) -> Self::Owl;
}
