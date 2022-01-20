use fastobo::error::CardinalityError;

/// The result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for this crate.
#[derive(Debug, Error)]
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
    /// # use fastobo::ast::*;
    ///
    /// let id = ClassIdent::from_str("MS:1000031").unwrap();
    /// let mut frame = TermFrame::new();
    /// frame.push(TermClause::UnionOf(Box::new(id)));
    ///
    /// let res = OboDoc::with_entities(frame).into_owl();
    /// assert!(matches!(res, Err(fastobo::error::Error::Cardinality(_))));
    /// ```
    #[error(transparent)]
    Cardinality(#[from] CardinalityError),
}
