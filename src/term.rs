use std::collections::BTreeSet;
use std::iter::FromIterator;

use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::datatype;
use crate::constants::property;

impl IntoOwlCtx for obo::TermFrame {
    type Owl = BTreeSet<owl::AnnotatedAxiom>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        // The ID of this frame translated to an IRI.
        let id = obo::Ident::from(self.id().clone().into_inner()).into_owl(ctx);

        // The translated axioms.
        let mut axioms: Self::Owl = BTreeSet::new();

        // Build the annotated class declaration.
        axioms.insert(owl::AnnotatedAxiom {
            annotation: BTreeSet::new(),
            axiom: owl::Axiom::from(owl::DeclareClass(owl::Class(id.clone()))),
        });

        // Add the original OBO ID as an annotation.
        // FIXME: maybe only do that if the term is not declared as being
        //        anonymous ?
        axioms.insert(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
            annotation_subject: id.clone(),
            annotation: owl::Annotation {
                annotation_property: ctx.build.annotation_property(property::obo_in_owl::ID),
                annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                    datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                    literal: Some(self.id().as_ref().to_string()),
                    lang: None,
                }),
            },
        }));

        // Group assertion clauses together:
        // - IntersectionOf(Option<RelationIdent>, ClassIdent),
        // - UnionOf(ClassIdent),

        // Convert remaining clauses to axioms.
        for line in self.into_iter() {
            if let Some(axiom) = line.into_inner().into_owl(ctx) {
                axioms.insert(axiom);
            }
        }

        // Return the axioms
        axioms
    }
}

impl IntoOwlCtx for obo::Line<obo::TermClause> {
    type Owl = Option<owl::AnnotatedAxiom>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        // FIXME: handle con
        self.into_inner().into_owl(ctx)
    }
}

impl IntoOwlCtx for obo::TermClause {
    type Owl = Option<owl::AnnotatedAxiom>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        match self {
            // IsAnonymous(bool),
            obo::TermClause::Name(name) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx.build.annotation_property(property::rdfs::LABEL),
                        annotation_value: name.into_owl(ctx).into(),
                    },
                }))
            }

            obo::TermClause::Namespace(ns) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx
                            .build
                            .annotation_property(property::obo_in_owl::HAS_OBO_NAMESPACE),
                        annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                            datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                            literal: Some(ns.to_string()),
                            lang: None,
                        }),
                    },
                }))
            }
            obo::TermClause::AltId(id) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx
                            .build
                            .annotation_property(property::obo_in_owl::HAS_ALTERNATIVE_ID),
                        annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                            datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                            literal: Some(id.to_string()),
                            lang: None,
                        }),
                    },
                }))
            }

            obo::TermClause::Def(desc, xrefs) => Some(owl::AnnotatedAxiom::new(
                owl::AnnotationAssertion::new(
                    ctx.current_frame.clone(),
                    owl::Annotation {
                        annotation_property: ctx
                            .build
                            .annotation_property(property::iao::DEFINITION),
                        annotation_value: desc.into_owl(ctx).into(),
                    },
                ),
                xrefs.into_owl(ctx),
            )),

            obo::TermClause::Comment(comment) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx.build.annotation_property(property::rdfs::COMMENT),
                        annotation_value: comment.into_owl(ctx).into(),
                    },
                }))
            }
            obo::TermClause::Subset(subset) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx
                            .build
                            .annotation_property(property::obo_in_owl::IN_SUBSET),
                        annotation_value: owl::AnnotationValue::IRI(
                            obo::Ident::from(subset).into_owl(ctx),
                        ),
                    },
                }))
            }

            obo::TermClause::Xref(xref) => Some(owl::AnnotatedAxiom::new(
                owl::Axiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: xref.clone().into_owl(ctx),
                }),
                BTreeSet::from_iter(xref.description().map(|desc| owl::Annotation {
                    annotation_property: ctx.build.annotation_property(property::rdfs::LABEL),
                    annotation_value: desc.clone().into_owl(ctx).into(),
                })),
            )),

            obo::TermClause::Builtin(_) => None,

            obo::TermClause::PropertyValue(pv) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: pv.into_owl(ctx),
                }))
            }

            obo::TermClause::IsA(supercls) => Some(owl::AnnotatedAxiom::from(owl::SubClassOf {
                super_class: owl::ClassExpression::Class(owl::Class(supercls.into_owl(ctx))),
                sub_class: owl::ClassExpression::Class(owl::Class(ctx.current_frame.clone())),
            })),

            // IntersectionOf(Option<RelationIdent>, ClassIdent),
            // UnionOf(ClassIdent),

            // FIXME: should be all grouped into a single axiom ?
            obo::TermClause::EquivalentTo(cid) => {
                Some(owl::AnnotatedAxiom::from(owl::EquivalentClasses(vec![
                    owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                    owl::ClassExpression::from(owl::Class(cid.into_owl(ctx))),
                ])))
            }

            obo::TermClause::DisjointFrom(cid) => {
                Some(owl::AnnotatedAxiom::from(owl::DisjointClasses(vec![
                    owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                    owl::ClassExpression::from(owl::Class(cid.into_owl(ctx))),
                ])))
            }

            // Relationship(RelationIdent, ClassIdent),
            obo::TermClause::IsObsolete(b) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx
                            .build
                            .annotation_property(property::owl::DEPRECATED),
                        annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                            datatype_iri: Some(ctx.build.iri(datatype::xsd::BOOLEAN)),
                            literal: Some(b.to_string()),
                            lang: None,
                        }),
                    },
                }))
            }

            obo::TermClause::ReplacedBy(id) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx
                            .build
                            .annotation_property(property::iao::REPLACED_BY),
                        annotation_value: owl::AnnotationValue::IRI(id.into_owl(ctx)),
                    },
                }))
            }

            obo::TermClause::Consider(id) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx
                            .build
                            .annotation_property(property::obo_in_owl::CONSIDER),
                        annotation_value: owl::AnnotationValue::IRI(
                            obo::Ident::from(id).into_owl(ctx),
                        ),
                    },
                }))
            }

            obo::TermClause::CreatedBy(c) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx.build.annotation_property(property::dc::CREATOR),
                        annotation_value: c.into_owl(ctx).into(),
                    },
                }))
            }

            obo::TermClause::CreationDate(dt) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx.build.annotation_property(property::dc::DATE),
                        annotation_value: dt.into_owl(ctx).into(),
                    },
                }))
            }

            _ => None,
        }
    }
}
