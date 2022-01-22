# `fastobo-owl` [![Star me](https://img.shields.io/github/stars/fastobo/fastobo-owl.svg?style=social&label=Star&maxAge=3600)](https://github.com/fastobo/fastobo-owl/stargazers)

*OWL language mapping for ontologies in the OBO flat file format 1.4*

[![Actions](https://img.shields.io/github/workflow/status/fastobo/fastobo-owl/Test?style=flat-square&maxAge=600)](https://github.com/fastobo/fastobo-owl/actions)
[![Codecov](https://img.shields.io/codecov/c/gh/fastobo/fastobo-owl/master.svg?style=flat-square&maxAge=600)](https://codecov.io/gh/fastobo/fastobo-owl)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square&maxAge=2678400)](https://choosealicense.com/licenses/mit/)
[![Source](https://img.shields.io/badge/source-GitHub-303030.svg?maxAge=2678400&style=flat-square)](https://github.com/fastobo/fastobo-owl/)
[![Crate](https://img.shields.io/crates/v/fastobo-owl.svg?maxAge=600&style=flat-square)](https://crates.io/crates/fastobo-owl)
[![Documentation](https://img.shields.io/badge/docs.rs-latest-4d76ae.svg?maxAge=2678400&style=flat-square)](https://docs.rs/fastobo-owl/)
[![Changelog](https://img.shields.io/badge/keep%20a-changelog-8A0707.svg?maxAge=2678400&style=flat-square)](https://github.com/fastobo/fastobo-owl/blob/master/CHANGELOG.md)
[![GitHub issues](https://img.shields.io/github/issues/fastobo/fastobo-owl.svg?style=flat-square)](https://github.com/fastobo/fastobo-owl/issues)
[![DOI](https://img.shields.io/badge/doi-10.7490%2Ff1000research.1117405.1-brightgreen?style=flat-square&maxAge=31536000)](https://f1000research.com/posters/8-1500)


## Overview

This library provides an implementation of the [OBO to OWL mappings](http://owlcollab.github.io/oboformat/doc/obo-syntax.html#5)
for the [OBO format version 1.4](https://owlcollab.github.io/oboformat/doc/GO.format.obo-1_4.html) ontology language.
It can be used to produce a semantically-equivalent OWL ontology from any
OBO ontology.


## Usage

Add `fastobo-owl` to the `[dependencies]` sections of your `Cargo.toml`
manifest:
```toml
[dependencies]
fastobo-owl = "0.1.2"
```

Then use the `IntoOwl` trait to convert an [`OboDoc`] into any OWL ontology
(the output type must implement the [`Default`], [`Ontology`] and [`MutableOntology`] traits).
Here's a how you could write a very simple script to load an OBO document
from a file, convert it to OWL, and write it to another file in OWL/XML syntax:

[`OboDoc`]: https://docs.rs/fastobo/latest/fastobo/ast/struct.OboDoc.html
[`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
[`Ontology`]: https://docs.rs/horned-owl/latest/horned_owl/model/trait.Ontology.html
[`MutableOntology`]: https://docs.rs/horned-owl/latest/horned_owl/model/trait.MutableOntology.html

```rust
extern crate fastobo;
extern crate fastobo_owl;

use fastobo_owl::IntoOwl;

// load an OBO ontology from a file
let obo = fastobo::from_file("tests/data/ms.obo").expect("failed to read OBO file");

// extract prefixes from the OBO header, so that they can be used
// to build abbreviated IRIs when serializing the OWL output
// (note: this contains OBO default prefixes such as xsd, rdf, or oboInOwl)
let prefixes = obo.prefixes();

// convert the ontology to OBO (the ontology type is implied by the later
// call to owx::writer::write which expects an `AxiomMappedOntology`)
let owl = obo.into_owl()
  .expect("failed to convert OBO to OWL");

// write the OWL ontology with abbreviated IRIs
let mut output = std::fs::File::create("tests/data/ms.owx").unwrap();
horned_owl::io::owx::writer::write(&mut output, &owl, Some(&prefixes));
```

## See also

* [`fastobo-syntax`](https://crates.io/crates/fastobo-syntax): Standalone `pest` parser
  for the OBO format version 1.4.
* [`fastobo`](https://crates.io/crates/fastobo): Abstract Syntax Tree and data
  structures for the OBO format version 1.4.
* [`fastobo-py`](https://pypi.org/project/fastobo/): Idiomatic Python bindings
  to the `fastobo` crate.
* [`fastobo-graphs`](https://crates.io/crates/fastobo-graphs): Data model and `serde`
  implementation of the OBO graphs specification, with conversion traits from and to OBO.
* [`fastobo-validator`](https://crates.io/crates/fastobo-validator): Standalone CLI
  to validate OBO files against the specification.

## Feedback

Found a bug ? Have an enhancement request ? Head over to the
[GitHub issue tracker](https://github.com/fastobo/fastobo-owl/issues) of the project if
you need to report or ask something. If you are filling in on a bug, please include as much
information as you can about the issue, and try to recreate the same bug in a simple, easily
reproducible situation.


## About

This project was developed by [Martin Larralde](https://github.com/althonos)
as part of a Master's Degree internship in the [BBOP team](http://berkeleybop.org/) of the
[Lawrence Berkeley National Laboratory](https://www.lbl.gov/), under the supervision of
[Chris Mungall](http://biosciences.lbl.gov/profiles/chris-mungall/). Cite this project as:

*Larralde M.* **Developing Python and Rust libraries to improve the ontology ecosystem**
*\[version 1; not peer reviewed\].* F1000Research 2019, 8(ISCB Comm J):1500 (poster)
([https://doi.org/10.7490/f1000research.1117405.1](https://doi.org/10.7490/f1000research.1117405.1))
