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
        let id = self.id().clone().into_inner().into_owl(ctx);

        // The translated axioms.
        let mut axioms: Self::Owl = BTreeSet::new();

        // Build the annotated class declaration.
        axioms.insert(owl::AnnotatedAxiom {
            ann: BTreeSet::new(),
            axiom: owl::Axiom::from(owl::DeclareClass(owl::Class(id.clone()))),
        });

        // Add the original OBO ID as an annotation.
        // FIXME: maybe only do that if the term is not declared as being
        //        anonymous ?
        axioms.insert(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
            subject: owl::Individual::from(&id),
            ann: owl::Annotation {
                ap: ctx.build.annotation_property(property::obo_in_owl::ID),
                av: owl::AnnotationValue::Literal(owl::Literal::Simple {
                    literal: self.id().as_ref().to_string(),
                }),
            },
        }));

        // Group some assertion clauses together
        let mut intersections: Vec<owl::ClassExpression> = Vec::new();
        let mut intersections_a: BTreeSet<owl::Annotation> = BTreeSet::new();
        let mut unions: Vec<owl::ClassExpression> = Vec::new();
        let mut unions_a: BTreeSet<owl::Annotation> = BTreeSet::new();

        // Convert remaining clauses to axioms.
        for line in self.into_iter() {
            if let Some(mut axiom) = line.into_owl(ctx) {
                if let owl::Axiom::EquivalentClasses(eq) = &axiom.axiom {
                    match &eq.0[1] {
                        owl::ClassExpression::ObjectIntersectionOf(objs) => {
                            intersections.append(&mut objs.clone());
                            intersections_a.append(&mut axiom.ann);
                        }
                        owl::ClassExpression::ObjectUnionOf(objs) => {
                            unions.append(&mut objs.clone());
                            unions_a.append(&mut axiom.ann);
                        }
                        _ => {
                            axioms.insert(axiom);
                        }
                    }
                } else {
                    axioms.insert(axiom);
                }
            }
        }

        // Add all intersections as a single `EquivalentClasses` axiom.
        if !intersections.is_empty() {
            axioms.insert(owl::AnnotatedAxiom::new(
                owl::Axiom::EquivalentClasses(owl::EquivalentClasses(vec![
                    owl::ClassExpression::Class(owl::Class(id.clone())),
                    owl::ClassExpression::ObjectIntersectionOf(intersections),
                ])),
                intersections_a,
            ));
        }

        // Add all unions as a single `EquivalentClasses` axiom.
        if !unions.is_empty() {
            axioms.insert(owl::AnnotatedAxiom::new(
                owl::Axiom::EquivalentClasses(owl::EquivalentClasses(vec![
                    owl::ClassExpression::Class(owl::Class(id.clone())),
                    owl::ClassExpression::ObjectUnionOf(unions),
                ])),
                unions_a,
            ));
        }

        // Return the axioms
        axioms
    }
}

impl IntoOwlCtx for obo::Line<obo::TermClause> {
    type Owl = Option<owl::AnnotatedAxiom>;
    fn into_owl(mut self, ctx: &mut Context) -> Self::Owl {
        // Take ownership of qualifiers list.
        let qualifiers = match self.qualifiers_mut() {
            Some(q) => std::mem::replace(q, obo::QualifierList::default()),
            None => obo::QualifierList::default(),
        };

        match self.into_inner() {
            //
            obo::TermClause::IntersectionOf(Some(rid), cid) => Some(owl::AnnotatedAxiom::new(
                owl::Axiom::EquivalentClasses(owl::EquivalentClasses(vec![
                    owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                    owl::ClassExpression::ObjectIntersectionOf(vec![ctx.rel_class_expression(
                        &qualifiers,
                        *rid,
                        *cid,
                    )]),
                ])),
                qualifiers.into_owl(ctx),
            )),
            //
            obo::TermClause::Relationship(rid, cid) => {
                let r_iri = rid.into_owl(ctx);
                if ctx.is_metadata_tag(&r_iri) {
                    Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                        subject: owl::Individual::Named(owl::NamedIndividual::from(
                            &ctx.current_frame,
                        )),
                        ann: owl::Annotation {
                            ap: owl::AnnotationProperty::from(r_iri),
                            av: owl::AnnotationValue::from(cid.into_owl(ctx)),
                        },
                    }))
                } else {
                    Some(owl::AnnotatedAxiom::new(
                        owl::Axiom::SubClassOf(owl::SubClassOf {
                            sub: owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                            sup: ctx.rel_class_expression(&qualifiers, *rid, *cid),
                        }),
                        qualifiers.into_owl(ctx),
                    ))
                }
            }
            //
            other => {
                if let Some(mut axiom) = other.into_owl(ctx) {
                    axiom.ann.append(&mut qualifiers.into_owl(ctx));
                    Some(axiom)
                } else {
                    None
                }
            }
        }
    }
}

impl IntoOwlCtx for obo::TermClause {
    type Owl = Option<owl::AnnotatedAxiom>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        match self {
            obo::TermClause::IsAnonymous(_) => None,

            obo::TermClause::Builtin(_) => None,

            obo::TermClause::Name(name) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::rdfs::LABEL),
                        av: name.into_owl(ctx).into(),
                    },
                }))
            }

            obo::TermClause::Namespace(ns) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx
                            .build
                            .annotation_property(property::obo_in_owl::HAS_OBO_NAMESPACE),
                        av: owl::AnnotationValue::Literal(owl::Literal::Simple {
                            literal: ns.to_string(),
                        }),
                    },
                }))
            }

            obo::TermClause::AltId(id) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx
                            .build
                            .annotation_property(property::obo_in_owl::HAS_ALTERNATIVE_ID),
                        av: owl::AnnotationValue::Literal(owl::Literal::Simple {
                            literal: id.to_string(),
                        }),
                    },
                }))
            }

            obo::TermClause::Def(def) => Some(def.into_owl(ctx)),

            obo::TermClause::Comment(comment) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::rdfs::COMMENT),
                        av: comment.into_owl(ctx).into(),
                    },
                }))
            }

            obo::TermClause::Subset(subset) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx
                            .build
                            .annotation_property(property::obo_in_owl::IN_SUBSET),
                        av: owl::AnnotationValue::IRI(subset.into_owl(ctx)),
                    },
                }))
            }

            obo::TermClause::Xref(xref) => Some(owl::AnnotatedAxiom::new(
                owl::Axiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: xref.clone().into_owl(ctx),
                }),
                BTreeSet::from_iter(xref.description().map(|desc| owl::Annotation {
                    ap: ctx.build.annotation_property(property::rdfs::LABEL),
                    av: desc.clone().into_owl(ctx).into(),
                })),
            )),

            obo::TermClause::PropertyValue(pv) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: pv.into_owl(ctx),
                }))
            }

            obo::TermClause::IsA(supercls) => Some(owl::AnnotatedAxiom::from(owl::SubClassOf {
                sup: owl::ClassExpression::Class(owl::Class(supercls.into_owl(ctx))),
                sub: owl::ClassExpression::Class(owl::Class(ctx.current_frame.clone())),
            })),

            // QUESTION: should be all grouped into a single axiom ?
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

            obo::TermClause::IsObsolete(b) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::owl::DEPRECATED),
                        av: owl::AnnotationValue::Literal(owl::Literal::Datatype {
                            datatype_iri: ctx.build.iri(datatype::xsd::BOOLEAN),
                            literal: b.to_string(),
                        }),
                    },
                }))
            }

            obo::TermClause::ReplacedBy(id) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::iao::REPLACED_BY),
                        av: owl::AnnotationValue::IRI(id.into_owl(ctx)),
                    },
                }))
            }

            obo::TermClause::Consider(id) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx
                            .build
                            .annotation_property(property::obo_in_owl::CONSIDER),
                        av: owl::AnnotationValue::IRI(id.into_owl(ctx)),
                    },
                }))
            }

            obo::TermClause::CreatedBy(c) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::dc::CREATOR),
                        av: c.into_owl(ctx).into(),
                    },
                }))
            }

            obo::TermClause::CreationDate(dt) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::dc::DATE),
                        av: dt.into_owl(ctx).into(),
                    },
                }))
            }

            obo::TermClause::Synonym(syn) => Some(syn.into_owl(ctx)),

            obo::TermClause::IntersectionOf(None, cid) => {
                Some(owl::AnnotatedAxiom::from(owl::EquivalentClasses(vec![
                    owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                    owl::ClassExpression::ObjectIntersectionOf(vec![owl::ClassExpression::from(
                        owl::Class(cid.into_owl(ctx)),
                    )]),
                ])))
            }

            obo::TermClause::UnionOf(cid) => {
                Some(owl::AnnotatedAxiom::from(owl::EquivalentClasses(vec![
                    owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                    owl::ClassExpression::ObjectUnionOf(vec![owl::ClassExpression::from(
                        owl::Class(cid.into_owl(ctx)),
                    )]),
                ])))
            }

            // These are handled on `Line<TermClause>::into_owl`
            obo::TermClause::Relationship(rid, cid) => Some(owl::AnnotatedAxiom::from(
                owl::Axiom::SubClassOf(owl::SubClassOf {
                    sub: owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                    sup: ctx.rel_class_expression(&obo::QualifierList::default(), *rid, *cid),
                }),
            )),

            obo::TermClause::IntersectionOf(Some(rid), cid) => Some(owl::AnnotatedAxiom::from(
                owl::Axiom::EquivalentClasses(owl::EquivalentClasses(vec![
                    owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                    owl::ClassExpression::ObjectIntersectionOf(vec![ctx.rel_class_expression(
                        &obo::QualifierList::default(),
                        *rid,
                        *cid,
                    )]),
                ])),
            )),
        }
    }
}
