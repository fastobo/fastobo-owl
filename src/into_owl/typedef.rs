use std::collections::BTreeSet;
use std::iter::FromIterator;

use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::datatype;
use crate::constants::property;

fn is_annotation_property(frame: &obo::TypedefFrame) -> bool {
    frame
        .iter()
        .any(|l| l.as_inner() == &obo::TypedefClause::IsMetadataTag(true))
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
                ann: BTreeSet::new(),
                axiom: owl::Axiom::from(owl::DeclareAnnotationProperty(id.clone().into())),
            });
            ctx.in_annotation = true;
        } else {
            // Object property.
            axioms.insert(owl::AnnotatedAxiom {
                ann: BTreeSet::new(),
                axiom: owl::Axiom::from(owl::DeclareObjectProperty(id.clone().into())),
            });
            ctx.in_annotation = false;
        }

        // Add the original OBO ID as an annotation.
        axioms.insert(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
            subject: owl::Individual::from(&id),
            ann: owl::Annotation {
                ap: ctx.build.annotation_property(property::obo_in_owl::ID),
                av: owl::AnnotationValue::Literal(owl::Literal::Simple {
                    literal: self.id().as_ref().to_string(),
                }),
            },
        }));

        // Add the typedef clauses.
        axioms.extend(self.into_iter().flat_map(|line| line.into_owl(ctx)));

        // Return the axioms
        axioms
    }
}

impl IntoOwlCtx for obo::Line<obo::TypedefClause> {
    type Owl = Option<owl::AnnotatedAxiom>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
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
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::rdfs::LABEL),
                        av: name.into_owl(ctx).into(),
                    },
                }))
            }
            obo::TypedefClause::Namespace(ns) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx
                            .build
                            .annotation_property(property::obo_in_owl::HAS_OBO_NAMESPACE),
                        av: owl::AnnotationValue::Literal(owl::Literal::Datatype {
                            datatype_iri: ctx.build.iri(datatype::xsd::STRING),
                            literal: ns.to_string(),
                        }),
                    },
                }))
            }
            obo::TypedefClause::AltId(id) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx
                            .build
                            .annotation_property(property::obo_in_owl::HAS_ALTERNATIVE_ID),
                        av: owl::AnnotationValue::Literal(owl::Literal::Datatype {
                            datatype_iri: ctx.build.iri(datatype::xsd::STRING),
                            literal: id.to_string(),
                        }),
                    },
                }))
            }
            obo::TypedefClause::Def(def) => Some(def.into_owl(ctx)),
            obo::TypedefClause::Comment(comment) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::rdfs::COMMENT),
                        av: comment.into_owl(ctx).into(),
                    },
                }))
            }
            obo::TypedefClause::Subset(subset) => {
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
            obo::TypedefClause::Synonym(syn) => Some(syn.into_owl(ctx)),
            obo::TypedefClause::Xref(xref) => {
                let annotation = xref.description().map(|desc| owl::Annotation {
                    ap: ctx.build.annotation_property(property::rdfs::LABEL),
                    av: desc.clone().into_owl(ctx).into(),
                });
                Some(owl::AnnotatedAxiom::new(
                    owl::Axiom::from(owl::AnnotationAssertion {
                        subject: owl::Individual::from(&ctx.current_frame),
                        ann: xref.into_owl(ctx),
                    }),
                    BTreeSet::from_iter(annotation),
                ))
            }
            obo::TypedefClause::PropertyValue(pv) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: pv.into_owl(ctx),
                }))
            }
            // obo::TypedefClause::Domain(ClassIdent),
            // obo::TypedefClause::Range(ClassIdent),
            obo::TypedefClause::Builtin(_) => None,
            obo::TypedefClause::HoldsOverChain(r1, r2) => {
                // holds_over_chain(Rel1-ID Rel2-ID Qualifiers) 	SubObjectPropertyOf(T(Qualifiers) ObjectPropertyChain( T(Rel1-ID) T(Rel2-ID) ) T(Rel-ID) )
                Some(owl::AnnotatedAxiom::from(owl::SubObjectPropertyOf {
                    sup: owl::ObjectPropertyExpression::from(&ctx.current_frame),
                    sub: owl::SubObjectPropertyExpression::ObjectPropertyChain(vec![
                        owl::ObjectPropertyExpression::ObjectProperty(r1.into_owl(ctx).into()),
                        owl::ObjectPropertyExpression::ObjectProperty(r2.into_owl(ctx).into()),
                    ]),
                }))
            }
            // obo::TypedefClause::IsAntiSymmetric(bool),
            // obo::TypedefClause::IsCyclic(bool),
            // obo::TypedefClause::IsReflexive(bool),
            obo::TypedefClause::IsSymmetric(false) => None,
            obo::TypedefClause::IsSymmetric(true) => {
                Some(owl::AnnotatedAxiom::from(owl::SymmetricObjectProperty(
                    owl::ObjectPropertyExpression::ObjectProperty(ctx.current_frame.clone().into()),
                )))
            }
            obo::TypedefClause::IsAsymmetric(false) => None,
            obo::TypedefClause::IsAsymmetric(true) => {
                Some(owl::AnnotatedAxiom::from(owl::AsymmetricObjectProperty(
                    owl::ObjectPropertyExpression::ObjectProperty(ctx.current_frame.clone().into()),
                )))
            }
            obo::TypedefClause::IsTransitive(false) => None,
            obo::TypedefClause::IsTransitive(true) => {
                Some(owl::AnnotatedAxiom::from(owl::TransitiveObjectProperty(
                    owl::ObjectPropertyExpression::ObjectProperty(ctx.current_frame.clone().into()),
                )))
            }
            // obo::TypedefClause::IsFunctional(bool),
            // obo::TypedefClause::IsInverseFunctional(bool),
            obo::TypedefClause::IsA(supercls) => {
                if ctx.in_annotation {
                    Some(owl::AnnotatedAxiom::from(owl::SubAnnotationPropertyOf {
                        sup: supercls.into_owl(ctx).into(),
                        sub: ctx.current_frame.clone().into(),
                    }))
                } else {
                    Some(owl::AnnotatedAxiom::from(owl::SubObjectPropertyOf {
                        sup: owl::ObjectPropertyExpression::ObjectProperty(
                            supercls.into_owl(ctx).into(),
                        ),
                        sub: owl::SubObjectPropertyExpression::ObjectPropertyExpression(
                            owl::ObjectPropertyExpression::ObjectProperty(
                                ctx.current_frame.clone().into(),
                            ),
                        ),
                    }))
                }
            }
            // obo::TypedefClause::IntersectionOf(RelationIdent),
            // obo::TypedefClause::UnionOf(RelationIdent),
            obo::TypedefClause::EquivalentTo(cls) => {
                if ctx.in_annotation {
                    Some(owl::AnnotatedAxiom::from(owl::EquivalentDataProperties(
                        vec![ctx.current_frame.clone().into(), cls.into_owl(ctx).into()],
                    )))
                } else {
                    Some(owl::AnnotatedAxiom::from(owl::EquivalentObjectProperties(
                        vec![
                            owl::ObjectPropertyExpression::from(&ctx.current_frame),
                            owl::ObjectPropertyExpression::ObjectProperty(cls.into_owl(ctx).into()),
                        ],
                    )))
                }
            }
            // obo::TypedefClause::DisjointFrom(RelationIdent),
            obo::TypedefClause::InverseOf(rid) => {
                Some(owl::AnnotatedAxiom::from(owl::InverseObjectProperties(
                    owl::ObjectProperty::from(&ctx.current_frame),
                    owl::ObjectProperty::from(rid.into_owl(ctx)),
                )))
            }
            obo::TypedefClause::TransitiveOver(rid) => {
                Some(owl::AnnotatedAxiom::from(owl::SubObjectPropertyOf {
                    sup: owl::ObjectPropertyExpression::from(&ctx.current_frame),
                    sub: owl::SubObjectPropertyExpression::ObjectPropertyChain(vec![
                        owl::ObjectPropertyExpression::from(&ctx.current_frame),
                        owl::ObjectPropertyExpression::ObjectProperty(rid.into_owl(ctx).into()),
                    ]),
                }))
            }
            // obo::TypedefClause::EquivalentToChain(RelationIdent, RelationIdent),
            // obo::TypedefClause::DisjointOver(RelationIdent),
            // obo::TypedefClause::Relationship(RelationIdent, RelationIdent),
            obo::TypedefClause::IsObsolete(b) => {
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
            obo::TypedefClause::ReplacedBy(id) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::iao::REPLACED_BY),
                        av: owl::AnnotationValue::IRI(id.into_owl(ctx)),
                    },
                }))
            }
            obo::TypedefClause::Consider(id) => {
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
            obo::TypedefClause::CreatedBy(c) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::dc::CREATOR),
                        av: c.into_owl(ctx).into(),
                    },
                }))
            }
            obo::TypedefClause::CreationDate(dt) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    subject: owl::Individual::from(&ctx.current_frame),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::dc::DATE),
                        av: dt.into_owl(ctx).into(),
                    },
                }))
            }
            // obo::TypedefClause::ExpandAssertionTo(QuotedString, XrefList),
            // obo::TypedefClause::ExpandExpressionTo(QuotedString, XrefList),
            obo::TypedefClause::IsMetadataTag(_) => None,
            obo::TypedefClause::IsClassLevel(_) => None,
            _ => unimplemented!("{}", &self),
        }
    }
}
