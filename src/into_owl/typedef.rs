use std::collections::BTreeSet;
use std::iter::FromIterator;

use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::datatype;
use crate::constants::property;

fn is_annotation_property(frame: &obo::TypedefFrame) -> bool {
    frame.iter()
        .any(|l| l.as_inner() ==  &obo::TypedefClause::IsClassLevel(true))
}

impl IntoOwlCtx for obo::TypedefFrame {
    type Owl = BTreeSet<owl::AnnotatedAxiom>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        // The ID of this frame translated to an IRI.
        let id = self.id().clone().into_inner().into_owl(ctx);

        // The translated axioms.
        let mut axioms: Self::Owl = BTreeSet::new();

        // Check if we translate as object or annotation property.
        if is_annotation_property(&self) {
            // Annotation property.
            axioms.insert(owl::AnnotatedAxiom {
                annotation: BTreeSet::new(),
                axiom: owl::Axiom::from(owl::DeclareAnnotationProperty(id.clone().into())),
            });
            ctx.in_annotation = true;
        } else {
            // Object property.
            axioms.insert(owl::AnnotatedAxiom {
                annotation: BTreeSet::new(),
                axiom: owl::Axiom::from(owl::DeclareObjectProperty(id.clone().into())),
            });
            ctx.in_annotation = false;
        }

        // Add the original OBO ID as an annotation.
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

        // Add the typedef clauses.
        for line in self.into_iter() {
            if let Some(mut axiom) = line.into_owl(ctx) {
                axioms.insert(axiom);
            }
        }


        // Return the axioms
        axioms
    }
}

impl IntoOwlCtx for obo::Line<obo::TypedefClause> {
    type Owl = Option<owl::AnnotatedAxiom>;
    fn into_owl(mut self, ctx: &mut Context) -> Self::Owl {
        self.into_inner().into_owl(ctx)
    }
}

impl IntoOwlCtx for obo::TypedefClause {
    type Owl = Option<owl::AnnotatedAxiom>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        match self {
            obo::TypedefClause::IsAnonymous(_) => None,
            obo::TypedefClause::Name(name) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx.build.annotation_property(property::rdfs::LABEL),
                        annotation_value: name.into_owl(ctx).into(),
                    },
                }))
            },
            obo::TypedefClause::Namespace(ns) => {
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
            },
            obo::TypedefClause::AltId(id) => {
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
            },
            obo::TypedefClause::Def(desc, xrefs) => {
                Some(owl::AnnotatedAxiom::new(
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
                ))
            },
            obo::TypedefClause::Comment(comment) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx.build.annotation_property(property::rdfs::COMMENT),
                        annotation_value: comment.into_owl(ctx).into(),
                    },
                }))
            },
            obo::TypedefClause::Subset(subset) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx
                            .build
                            .annotation_property(property::obo_in_owl::IN_SUBSET),
                        annotation_value: owl::AnnotationValue::IRI(subset.into_owl(ctx)),
                    },
                }))
            },
            obo::TypedefClause::Synonym(syn) => Some(syn.into_owl(ctx)),
            obo::TypedefClause::Xref(xref) => {
                let annotation = xref.description().map(|desc| owl::Annotation {
                    annotation_property: ctx.build.annotation_property(property::rdfs::LABEL),
                    annotation_value: desc.clone().into_owl(ctx).into(),
                });
                Some(owl::AnnotatedAxiom::new(
                   owl::Axiom::from(owl::AnnotationAssertion {
                       annotation_subject: ctx.current_frame.clone(),
                       annotation: xref.into_owl(ctx),
                   }),
                   BTreeSet::from_iter(annotation),
               ))
            },
            obo::TypedefClause::PropertyValue(pv) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: pv.into_owl(ctx),
                }))
            },
            // obo::TypedefClause::Domain(ClassIdent),
            // obo::TypedefClause::Range(ClassIdent),
            obo::TypedefClause::Builtin(_) => None,
            // obo::TypedefClause::HoldsOverChain(RelationIdent, RelationIdent),
            // obo::TypedefClause::IsAntiSymmetric(bool),
            // obo::TypedefClause::IsCyclic(bool),
            // obo::TypedefClause::IsReflexive(bool),
            // obo::TypedefClause::IsSymmetric(bool),
            // obo::TypedefClause::IsAsymmetric(bool),
            // obo::TypedefClause::IsTransitive(bool),
            // obo::TypedefClause::IsFunctional(bool),
            // obo::TypedefClause::IsInverseFunctional(bool),
            obo::TypedefClause::IsA(supercls) => {
                Some(if ctx.in_annotation {
                    owl::AnnotatedAxiom::from(owl::SubAnnotationPropertyOf {
                        super_property: supercls.into_owl(ctx).into(),
                        sub_property: ctx.current_frame.clone().into(),
                    })
                } else {
                    owl::AnnotatedAxiom::from(owl::SubObjectPropertyOf {
                        super_property: owl::ObjectPropertyExpression::ObjectProperty(
                            supercls.into_owl(ctx).into()
                        ),
                        sub_property: owl::SubObjectPropertyExpression::ObjectPropertyExpression(
                            owl::ObjectPropertyExpression::ObjectProperty(ctx.current_frame.clone().into())
                        ),
                    })
                })
            },
            // obo::TypedefClause::IntersectionOf(RelationIdent),
            // obo::TypedefClause::UnionOf(RelationIdent),
            // obo::TypedefClause::EquivalentTo(RelationIdent),
            // obo::TypedefClause::DisjointFrom(RelationIdent),
            // obo::TypedefClause::InverseOf(RelationIdent),
            // obo::TypedefClause::TransitiveOver(RelationIdent),
            // obo::TypedefClause::EquivalentToChain(RelationIdent, RelationIdent),
            // obo::TypedefClause::DisjointOver(RelationIdent),
            // obo::TypedefClause::Relationship(RelationIdent, RelationIdent),
            obo::TypedefClause::IsObsolete(b) => {
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
            },
            obo::TypedefClause::ReplacedBy(id) => {
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
            obo::TypedefClause::Consider(id) => {
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
            obo::TypedefClause::CreatedBy(c) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx.build.annotation_property(property::dc::CREATOR),
                        annotation_value: c.into_owl(ctx).into(),
                    },
                }))
            },
            obo::TypedefClause::CreationDate(dt) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: ctx.current_frame.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx.build.annotation_property(property::dc::DATE),
                        annotation_value: dt.into_owl(ctx).into(),
                    },
                }))
            },
            // obo::TypedefClause::ExpandAssertionTo(QuotedString, XrefList),
            // obo::TypedefClause::ExpandExpressionTo(QuotedString, XrefList),
            // obo::TypedefClause::IsMetadataTag(bool),
            obo::TypedefClause::IsClassLevel(_) => None,
            _ => unimplemented!(),
        }
    }
}
