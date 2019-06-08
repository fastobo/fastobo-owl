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

        // Group some assertion clauses together
        // FIXME: merge annotations as well !
        let mut intersections: Vec<owl::ClassExpression> = Vec::new();
        let mut intersections_a: BTreeSet<owl::Annotation> = BTreeSet::new();
        let mut unions: Vec<owl::ClassExpression> = Vec::new();
        let mut unions_a: BTreeSet<owl::Annotation> = BTreeSet::new();

        // Convert remaining clauses to axioms.
        for line in self.into_iter() {
            if let Some(mut axiom) = line.into_owl(ctx) {
                if let owl::Axiom::EquivalentClasses(eq) = &axiom.axiom {
                    match &eq.0[1] {
                        owl::ClassExpression::ObjectIntersectionOf { o, .. } => {
                            intersections.append(&mut o.clone());
                            intersections_a.append(&mut axiom.annotation);
                        }
                        owl::ClassExpression::ObjectUnionOf { o, .. } => {
                            unions.append(&mut o.clone());
                            unions_a.append(&mut axiom.annotation);
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
                    owl::ClassExpression::ObjectIntersectionOf { o: intersections },
                ])),
                intersections_a,
            ));
        }

        // Add all unions as a single `EquivalentClasses` axiom.
        if !unions.is_empty() {
            axioms.insert(owl::AnnotatedAxiom::new(
                owl::Axiom::EquivalentClasses(owl::EquivalentClasses(vec![
                    owl::ClassExpression::Class(owl::Class(id.clone())),
                    owl::ClassExpression::ObjectUnionOf { o: unions },
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
        // FIXME: handle qualifiers that have semantic value other than
        //        simple annotations.
        if let Some(mut axiom) = self.into_inner().into_owl(ctx) {
            match axiom.axiom {
                // Transform the class expression of a translated `relationship`
                owl::Axiom::SubClassOf(owl::SubClassOf {
                    sub_class: owl::ClassExpression::Class(sub),
                    super_class:
                        owl::ClassExpression::ObjectSomeValuesFrom {
                            o: owl::ObjectPropertyExpression::ObjectProperty(r),
                            ce: cls,
                        },
                }) => {
                    axiom.axiom = owl::Axiom::SubClassOf(owl::SubClassOf {
                        sub_class: owl::ClassExpression::Class(sub),
                        super_class: ctx.rel_class_expression(&qualifiers, r, cls),
                    });
                }
                // Transform the class expression of a translated `intersection_of`
                owl::Axiom::EquivalentClasses(ref v) => {
                    if let owl::ClassExpression::ObjectIntersectionOf { o, .. } = &v.0[1] {
                        if let owl::ClassExpression::ObjectSomeValuesFrom {
                            o: owl::ObjectPropertyExpression::ObjectProperty(r),
                            ce: cls,
                        } = &o[0]
                        {
                            axiom.axiom = owl::Axiom::from(owl::EquivalentClasses(vec![
                                v.0[0].clone(),
                                owl::ClassExpression::ObjectIntersectionOf {
                                    o: vec![ctx.rel_class_expression(
                                        &qualifiers,
                                        r.clone(),
                                        cls.clone(),
                                    )],
                                },
                            ]));
                        }
                    }
                }
                // No transformation needed otherwise
                _ => (),
            };

            axiom.annotation.append(&mut qualifiers.into_owl(ctx));
            Some(axiom)
        } else {
            None
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
                        annotation_value: owl::AnnotationValue::IRI(subset.into_owl(ctx)),
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
                        annotation_value: owl::AnnotationValue::IRI(id.into_owl(ctx)),
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

            obo::TermClause::Synonym(syn) => Some(syn.into_owl(ctx)),

            // FIXME: requires qualifier check --> do that on line level.
            obo::TermClause::IntersectionOf(rid, cid) => {
                Some(owl::AnnotatedAxiom::from(owl::EquivalentClasses(vec![
                    owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                    owl::ClassExpression::ObjectIntersectionOf {
                        o: vec![match rid {
                            None => owl::ClassExpression::from(owl::Class(cid.into_owl(ctx))),
                            Some(r) => owl::ClassExpression::ObjectSomeValuesFrom {
                                o: owl::ObjectPropertyExpression::ObjectProperty(
                                    owl::ObjectProperty(r.into_owl(ctx)),
                                ),
                                ce: Box::new(owl::ClassExpression::from(owl::Class(
                                    cid.into_owl(ctx),
                                ))),
                            },
                        }],
                    },
                ])))
            }

            obo::TermClause::UnionOf(cid) => {
                Some(owl::AnnotatedAxiom::from(owl::EquivalentClasses(vec![
                    owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                    owl::ClassExpression::ObjectUnionOf {
                        o: vec![owl::ClassExpression::from(owl::Class(cid.into_owl(ctx)))],
                    },
                ])))
            }

            obo::TermClause::Relationship(rid, cid) => {
                Some(owl::AnnotatedAxiom::from(owl::SubClassOf {
                    sub_class: owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                    super_class: owl::ClassExpression::ObjectSomeValuesFrom {
                        o: owl::ObjectPropertyExpression::ObjectProperty(rid.into_owl(ctx).into()),
                        ce: Box::new(owl::ClassExpression::Class(cid.into_owl(ctx).into())),
                    },
                }))
            }
        }
    }
}
