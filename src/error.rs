use fastobo::error::CardinalityError;
use fastobo::error::SyntaxError;

/// The result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for this crate.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// An error caused by a clause appearing an invalid number of times.
    ///
    /// For instance, OBO frames have `union_of` clauses, that can not appear
    /// only once: they must appear zero, or more than two times. Having a
    /// frame with a single `union_of` clause will error when attempting to
    /// translate the whole document.
    ///
    /// # Example:
    /// ```rust
    /// # use std::str::FromStr;
    /// # use fastobo::ast::*;
    /// # use horned_owl::ontology::set::SetOntology;
    /// use fastobo_owl::IntoOwl;
    ///
    /// let mut frame = TermFrame::new(ClassIdent::from(PrefixedIdent::new("TST", "001")));
    /// let id = Box::new(ClassIdent::from(PrefixedIdent::new("TST", "002")));
    /// frame.push(Line::from(TermClause::UnionOf(id)));
    ///
    /// let doc = OboDoc::with_entities(vec![EntityFrame::from(frame)]);
    /// let res = doc.into_owl::<SetOntology>();
    /// assert!(matches!(res, Err(fastobo_owl::Error::Cardinality(_))));
    /// ```
    #[error(transparent)]
    Cardinality(#[from] CardinalityError),

    #[error(transparent)]
    /// An error caused by an element in invalid syntax.
    ///
    /// This can be raised while building IRI and Version IRI for an OWL
    /// ontology from an OBO `ontology` header clause, which may contain
    /// invalid data.
    ///
    Syntax(#[from] SyntaxError),
}
